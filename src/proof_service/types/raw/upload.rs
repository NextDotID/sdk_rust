// type ProofUploadRequest struct {
// 	Action        types.Action            `json:"action"`
// 	Platform      types.Platform          `json:"platform"`
// 	Identity      string                  `json:"identity"`
// 	ProofLocation string                  `json:"proof_location"`
// 	PublicKey     string                  `json:"public_key"`
// 	Uuid          string                  `json:"uuid"`
// 	CreatedAt     string                  `json:"created_at"`
// 	Extra         ProofUploadRequestExtra `json:"extra"`
// }
// type ProofUploadRequestExtra struct {
// 	Signature               string `json:"signature"`
// 	EthereumWalletSignature string `json:"wallet_signature"`
// }

use serde::{Deserialize, Serialize};

use crate::proof_service::{Action, Platform};

#[derive(Serialize)]
pub struct Request {
    pub action: Action,
    pub platform: Platform,
    pub identity: String,
    pub proof_location: String,
    pub public_key: String,
    pub uuid: String,
    pub created_at: String,
    pub extra: RequestExtra,
}

#[derive(Serialize)]
pub struct RequestExtra {
    pub signature: Option<String>,
    pub wallet_signature: Option<String>,
}

/// No response data. Only `201` siganl.
#[derive(Deserialize)]
pub struct Response {}
