use chrono::NaiveDateTime;
use http::Method;
use serde_json::Value;

use super::{
    types::{
        raw::{PayloadRequest, PayloadResponse, QueryResponse, UploadRequest},
        KVSingleProof,
    },
    Endpoint,
};
use crate::{
    proof_service::{Action, Platform},
    types::{Error, Result},
    util::{base64_encode, crypto::Secp256k1KeyPair, hex_encode, http::request, ts_to_naive},
};

pub struct KVProcedure {
    pub endpoint: Endpoint,
    pub action: Action,
    pub avatar: Secp256k1KeyPair,
    pub platform: Platform,
    pub identity: String,
    pub patch: Value,

    created_at: Option<NaiveDateTime>,
    uuid: Option<String>,
    pub sign_payload: Option<String>,
    signature: Option<Vec<u8>>,
}

impl KVProcedure {
    /// Start a new KVService modification procedure.
    pub fn new(
        endpoint: Endpoint,
        action: Action,
        avatar: Secp256k1KeyPair,
        platform: Platform,
        identity: &str,
        patch: Value,
    ) -> Self {
        KVProcedure {
            endpoint,
            action,
            avatar,
            platform,
            identity: identity.to_string(),
            patch,
            created_at: None,
            uuid: None,
            sign_payload: None,
            signature: None,
        }
    }

    /// Request for signature payloads from KVService.
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use nextid_sdk::kv_service::KVProcedure;
    /// # use nextid_sdk::kv_service::Endpoint;
    /// # use nextid_sdk::proof_service::{Action, Platform};
    /// # use nextid_sdk::util::crypto::Secp256k1KeyPair;
    /// # use serde_json::json;
    /// # let avatar = Secp256k1KeyPair::from_pk_hex("0x020d2ee3a597c24c66717dba01d7d14cb55e307834fe23428bd85c64249111f08a").unwrap();
    /// let mut procedure = KVProcedure::new(Endpoint::Staging, Action::Create, avatar, Platform::Twitter, "yeiwb", json!({"test": "abc123"}));
    /// assert_eq!((), procedure.get_payload().await.unwrap());
    /// # assert!(procedure.sign_payload.unwrap().len() > 0)
    /// # }
    /// ```
    pub async fn get_payload(&mut self) -> Result<()> {
        let url = self
            .endpoint
            .uri::<Vec<(String, String)>, _, _>("v1/kv/payload", vec![])?;
        let avatar_pubkey_hex = format!("0x{}", hex_encode(&self.avatar.pk.serialize_compressed()));
        let request_body = PayloadRequest {
            avatar: &avatar_pubkey_hex,
            platform: &self.platform,
            identity: &self.identity,
            patch: &self.patch,
        };
        let response: PayloadResponse = request(
            Method::POST,
            &url,
            serde_json::to_vec(&request_body)?.into(),
        )
        .await?;

        self.uuid = Some(response.uuid);
        self.created_at = Some(ts_to_naive(response.created_at, 0));
        self.sign_payload = Some(response.sign_payload);

        Ok(())
    }

    /// Submit the KV patch to KVService.
    /// If success, returns all KVs under this avatar.
    pub async fn submit(&mut self, avatar_signature: Vec<u8>) -> Result<Vec<KVSingleProof>> {
        // Valiadte signature locally before requesting.
        let recovered = Secp256k1KeyPair::recover_from_personal_signature(
            &avatar_signature,
            self.sign_payload.as_ref().unwrap(),
        )?;
        if recovered.pk != self.avatar.pk {
            return Err(Error::ServerError(
                "KVProcedure.submit(): Pubkey recovered from signature mismatches `self.avatar`."
                    .into(),
            ));
        }
        self.signature = Some(avatar_signature);

        let url = self
            .endpoint
            .uri::<Vec<(String, String)>, _, _>("v1/kv", vec![])?;
        let avatar = format!("0x{}", hex_encode(&self.avatar.pk.serialize_compressed()));
        let signature = base64_encode(&self.signature.clone().unwrap());
        let request_body = UploadRequest {
            avatar: &avatar,
            platform: &self.platform,
            identity: &self.identity,
            signature: &signature,
            uuid: self.uuid.as_ref().unwrap(),
            created_at: self.created_at.as_ref().unwrap().timestamp(),
            patch: &self.patch,
        };
        let response: QueryResponse = request(
            Method::POST,
            &url,
            serde_json::to_vec(&request_body)?.into(),
        )
        .await?;

        Ok(response.proofs)
    }
}
