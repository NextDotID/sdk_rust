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

#[derive(Serialize, Deserialize)]
pub struct ProofPayloadExtra {
    /// Ethereum wallet signature
    #[serde(rename = "wallet_signature")]
    pub ethereum_wallet_signature: Vec<u8>,
}

/// ProofChain modification procedure instance.
pub struct ProofProcedure {
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
    /// assert_eq!((), procedure.get_payload().await.unwrap());
    /// # assert!(procedure.sign_payload.unwrap().len() > 0)
    /// # }
    /// ```
    pub async fn get_payload(&mut self) -> Result<()> {
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

        self.uuid = Some(response.uuid);
        self.created_at = Some(util::ts_string_to_naive(&response.created_at)?);
        self.sign_payload = Some(response.sign_payload.clone());
        self.post_content = Some(response.post_content.clone());

        Ok(())
    }

    /// Submit this ProofChain modification to ProofService.
    /// If `self.platform == Ethereum && self.action == Create`, `avatar_signature` and `ethereum_signature` must both be provided.
    /// If `self.platform == Ethereum && self.action == Delete`, either of `avatar_signature` or `ethereum_signature` should be provided.
    /// Otherwise, leave these `None`.
    pub async fn submit(
        &mut self,
        proof_location: String,
        avatar_signature: Option<Vec<u8>>,
        ethereum_signature: Option<Vec<u8>>,
    ) -> Result<()> {
        self.proof_location = Some(proof_location);
        let upload_extra = self.local_validate(avatar_signature, ethereum_signature)?;

        // Local validation passed. Requesting remote ProofService.
        let url = self
            .endpoint
            .uri::<Vec<(String, String)>, _, _>("v1/proof", vec![])?;
        let request_body = UploadRequest {
            action: self.action,
            platform: self.platform,
            identity: self.identity.clone(),
            proof_location: self.proof_location.clone().unwrap(),
            public_key: hex_encode(&self.avatar.pk.serialize_compressed()),
            uuid: self
                .uuid
                .clone()
                .expect("UUID must be available at this moment."),
            created_at: self
                .created_at
                .expect("creatd_at must be available at this moment.")
                .timestamp()
                .to_string(),
            extra: upload_extra,
        };
        request::<UploadResponse>(
            Method::POST,
            &url,
            serde_json::to_vec(&request_body)?.into(),
        )
        .await?;

        Ok(())
    }

    /// Validate signature locally (if we can).
    fn local_validate(
        &mut self,
        avatar_signature: Option<Vec<u8>>,
        ethereum_signature: Option<Vec<u8>>,
    ) -> Result<UploadExtra> {
        // Local validation only make sense on Platform::Ethereum.
        if self.platform != Platform::Ethereum {
            return Ok(UploadExtra {
                signature: None,
                wallet_signature: None,
            });
        }

        match self.action {
            Action::Create => {
                // For creation, both of the signatures are needed.
                // Valiadte avatar sig locally before requesting.
                self.local_validate_avatar_sig(avatar_signature.as_ref())
                    .and(self.local_validate_eth_sig(ethereum_signature.as_ref()))?;
            }
            Action::Delete => {
                // For Ethereum deletion, only one of the valid signature provided should be OK.
                self.local_validate_avatar_sig(avatar_signature.as_ref())
                    .or(self.local_validate_eth_sig(ethereum_signature.as_ref()))?;
            }
        }

        // If success, modify myself.
        self.signature = avatar_signature;
        self.extra = Some(ProofPayloadExtra {
            ethereum_wallet_signature: ethereum_signature.unwrap(),
        });
        Ok(UploadExtra {
            wallet_signature: self
                .extra
                .as_ref()
                .map(|e| base64_encode(&e.ethereum_wallet_signature)),
            signature: self.signature.as_ref().map(|sig| base64_encode(sig)),
        })
    }

    /// Validate avatar signature.
    fn local_validate_avatar_sig(&self, avatar_signature: Option<&Vec<u8>>) -> Result<()> {
        if avatar_signature.is_none() {
            return Err(Error::ServerError(
                "ProofProcedure.local_validate_avatar_sig(): Avatar signature required.".into(),
            ));
        }

        let recovered = Secp256k1KeyPair::recover_from_personal_signature(
            avatar_signature.unwrap(),
            self.sign_payload.as_ref().unwrap(),
        )?;
        if recovered.pk != self.avatar.pk {
            Err(Error::ServerError(
                "ProofProcedure.local_validate_avatar_sig(): Pubkey recovered from signature mismatches `self.avatar`.".into(),
            ))
        } else {
            Ok(())
        }
    }

    /// Validate ethereum signature.
    fn local_validate_eth_sig(&self, ethereum_signature: Option<&Vec<u8>>) -> Result<()> {
        if ethereum_signature.is_none() {
            return Err(Error::ServerError(
                "ProofProcedure.local_validate_eth_sig(): Ethereum wallet signature required."
                    .into(),
            ));
        }

        let eth_sig = ethereum_signature.unwrap();
        let recovered = Secp256k1KeyPair::recover_from_personal_signature(
            &eth_sig,
            &self.sign_payload.clone().unwrap(),
        )?;
        let expected_address = hex_decode(&self.identity)?;
        let recovered_address: Vec<u8> = eth_address_from_public_key(&recovered.pk).into();
        if expected_address != recovered_address {
            Err(Error::ServerError(format!(
                "ProofProcedure.local_validate_eth_sig(): Ethereum address and signatures mismatch."
            )))
        } else {
            Ok(())
        }
    }
}
