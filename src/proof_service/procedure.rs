use super::{
    types::raw::payload::{Request, Response},
    Action, Endpoint, Platform,
};
use crate::{
    types::{Error, Result},
    util::{self, crypto::Secp256k1KeyPair, http::request},
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
    pub async fn payload(&mut self) -> Result<()> {
        if self.status != ProcedureStatus::Created {
            return Err(Error::ServerError(format!(
                "ProofProcedure.payload(): status should be `Created`. Current: {}",
                self.status
            )));
        }

        let url = self
            .endpoint
            .uri::<Vec<(String, String)>, _, _>("v1/proof/payload", vec![])?;
        let request_body = Request {
            action: self.action.clone(),
            platform: self.platform.clone(),
            identity: self.identity.clone(),
            public_key: util::hex_encode(&self.avatar.pk.serialize()),
            extra: None,
        };
        let response: Response = request(
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
    pub fn fill_signature(
        &mut self,
        signature: Vec<u8>,
        ethereum_signature: Option<Vec<u8>>,
    ) -> Result<()> {
        if self.status != ProcedureStatus::PayloadGenerated {
            return Err(Error::ServerError(format!(
                "ProofProcedure.fill_signature(): status should be `PayloadGenerated`. Current: {}",
                self.status
            )));
        }

        if self.platform == Platform::Ethereum && ethereum_signature.is_none() {
            return Err(Error::ServerError(
                "ProofProcedure.fill_signature(): Ethereum wallet signature required.".into(),
            ));
        }

        // Valiadte sig locally
        let recovered = Secp256k1KeyPair::recover_from_personal_signature(
            &signature,
            self.sign_payload.as_ref().unwrap(),
        )?;
        if recovered.pk != self.avatar.pk {
            return Err(Error::ServerError(
                "ProofProcedure.fill_signature(): Pubkey recovered from signature mismatches `self.avatar`.".into(),
            ));
        }

        todo!()
    }
}