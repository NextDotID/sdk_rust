use super::{
    types::raw::payload::{Request as PayloadRequest, Response as PayloadResponse},
    types::raw::upload::{
        Request as UploadRequest, RequestExtra as UploadExtra, Response as UploadResponse,
    },
    Action, Endpoint, Platform,
};
use crate::{
    types::{Error, Result},
    util::{
        self, base64_encode, crypto::Secp256k1KeyPair, eth_address_from_public_key, hex_decode,
        hex_encode, http::request,
    },
};
use chrono::NaiveDateTime;
use http::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::Display;

/// FST of ProofChain modification procedure.
#[derive(Display, PartialEq, Eq)]
pub enum ProcedureStatus {
    /// Just created, haven't communicate with ProofService.
    Created,
    /// Asked ProofService for sign payload, waiting for signature generated.
    PayloadGenerated,
    /// Signature is set, haven't post it to ProofService.
    Signed,
    /// ProofService accepted this modification.
    Committed,
}

#[derive(Serialize, Deserialize)]
pub struct ProofPayloadExtra {
    /// Ethereum wallet signature
    #[serde(rename = "wallet_signature")]
    pub ethereum_wallet_signature: Vec<u8>,
}

/// ProofChain modification procedure instance.
pub struct ProofProcedure {
    /// Workss as an FST: Which step now?
    pub status: ProcedureStatus,

    pub endpoint: Endpoint,
    pub action: Action,
    pub avatar: Secp256k1KeyPair,
    pub platform: Platform,
    pub identity: String,

    extra: Option<ProofPayloadExtra>,
    signature: Option<Vec<u8>>,
    uuid: Option<String>,
    created_at: Option<NaiveDateTime>,
    proof_location: Option<String>,

    pub post_content: Option<HashMap<String, String>>,
    pub sign_payload: Option<String>,
}

impl ProofProcedure {
    /// Start a new ProofService modification procedure.
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use nextid_sdk::proof_service::ProofProcedure;
    /// # use nextid_sdk::proof_service::{Endpoint, Action, Platform};
    /// # use nextid_sdk::util::crypto::Secp256k1KeyPair;
    /// # let mut rng = rand::rngs::OsRng;
    /// # let avatar_keypair = Secp256k1KeyPair::generate(&mut rng);
    /// let mut procedure = ProofProcedure::new(Endpoint::Staging, Action::Create, avatar_keypair, Platform::Twitter, "example");
    /// # }
    /// ```
    pub fn new(
        endpoint: Endpoint,
        action: Action,
        avatar: Secp256k1KeyPair,
        platform: Platform,
        identity: &str,
    ) -> Self {
        Self {
            status: ProcedureStatus::Created,
            endpoint,
            action,
            avatar,
            platform,
            identity: identity.to_string(),
            extra: None,
            signature: None,
            post_content: None,
            sign_payload: None,
            uuid: None,
            created_at: None,
            proof_location: None,
        }
    }

    /// Request for signature payloads and post content from ProofService.
    /// Will fill `self`'s `sign_payload`, `post_content`, `uuid` and `created_at`.
    /// And change `self.status` to [PayloadGenerated](ProcedureStatus::PayloadGenerated).
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use nextid_sdk::proof_service::ProofProcedure;
    /// # use nextid_sdk::proof_service::{Endpoint, Action, Platform};
    /// # use nextid_sdk::util::crypto::Secp256k1KeyPair;
    /// # let mut rng = rand::rngs::OsRng;
    /// # let avatar = Secp256k1KeyPair::generate(&mut rng);
    /// let mut procedure = ProofProcedure::new(Endpoint::Staging, Action::Create, avatar, Platform::Twitter, "example");
    /// assert_eq!((), procedure.payload().await.unwrap());
    /// # assert!(procedure.sign_payload.unwrap().len() > 0)
    /// # }
    /// ```
    pub async fn get_payload(&mut self) -> Result<()> {
        if self.status != ProcedureStatus::Created {
            return Err(Error::ServerError(format!(
                "ProofProcedure.payload(): status should be `Created`. Current: {}",
                self.status
            )));
        }

        let url = self
            .endpoint
            .uri::<Vec<(String, String)>, _, _>("v1/proof/payload", vec![])?;
        let request_body = PayloadRequest {
            action: self.action,
            platform: self.platform,
            identity: self.identity.clone(),
            public_key: util::hex_encode(&self.avatar.pk.serialize()),
            extra: None,
        };
        let response: PayloadResponse = request(
            Method::POST,
            &url,
            serde_json::to_vec(&request_body)?.into(),
        )
        .await?;

        self.status = ProcedureStatus::PayloadGenerated;
        self.uuid = Some(response.uuid);
        self.created_at = Some(util::ts_string_to_naive(&response.created_at)?);
        self.sign_payload = Some(response.sign_payload.clone());
        self.post_content = Some(response.post_content.clone());

        Ok(())
    }

    /// Fulfill signature generated by user, requested by frontend of your app.
    /// And submit this ProofChain modification to ProofService.
    pub async fn submit(
        &mut self,
        proof_location: String,
        avatar_signature: Vec<u8>,
        ethereum_signature: Option<Vec<u8>>,
    ) -> Result<()> {
        if self.status != ProcedureStatus::PayloadGenerated {
            return Err(Error::ServerError(format!(
                "ProofProcedure.fill_signature(): status should be `PayloadGenerated`. Current: {}",
                self.status
            )));
        }

        // Valiadte sig locally before requesting.
        let recovered = Secp256k1KeyPair::recover_from_personal_signature(
            &avatar_signature,
            self.sign_payload.as_ref().unwrap(),
        )?;
        if recovered.pk != self.avatar.pk {
            return Err(Error::ServerError(
                "ProofProcedure.fill_signature(): Pubkey recovered from signature mismatches `self.avatar`.".into(),
            ));
        }
        // Validate ETH signature locally before requesting.
        if self.platform == Platform::Ethereum {
            if ethereum_signature.is_none() {
                return Err(Error::ServerError(
                    "ProofProcedure.fill_signature(): Ethereum wallet signature required.".into(),
                ));
            }

            let eth_sig = ethereum_signature.clone().unwrap();
            let recovered = Secp256k1KeyPair::recover_from_personal_signature(
                &eth_sig,
                &self.sign_payload.clone().unwrap(),
            )?;
            let expected_address = hex_decode(&self.identity)?;
            let recovered_address: Vec<u8> = eth_address_from_public_key(&recovered.pk).into();
            if expected_address != recovered_address {
                return Err(Error::ServerError(format!(
                    "ProofProcedure.fill_signature(): Ethereum address and signatures mismatch."
                )));
            }
            self.extra = Some(ProofPayloadExtra {
                ethereum_wallet_signature: eth_sig.clone(),
            });
        }

        // Local validation passed. Requesting remote ProofService.
        self.proof_location = Some(proof_location);
        self.signature = Some(avatar_signature);
        let url = self
            .endpoint
            .uri::<Vec<(String, String)>, _, _>("v1/proof", vec![])?;
        let request_body = UploadRequest {
            action: self.action,
            platform: self.platform,
            identity: self.identity.clone(),
            proof_location: self.proof_location.clone().unwrap(),
            public_key: hex_encode(&self.avatar.pk.serialize_compressed()),
            uuid: self.uuid.clone().expect("UUID must be available."),
            created_at: self
                .created_at
                .expect("creatd_at must be available.")
                .timestamp()
                .to_string(),
            extra: UploadExtra {
                wallet_signature: if self.platform == Platform::Ethereum {
                    Some(base64_encode(&ethereum_signature.unwrap()))
                } else {
                    None
                },
                signature: base64_encode(&self.signature.clone().unwrap()),
            },
        };
        let _: UploadResponse = request(
            Method::POST,
            &url,
            serde_json::to_vec(&request_body)?.into(),
        )
        .await?;

        Ok(())
    }
}
