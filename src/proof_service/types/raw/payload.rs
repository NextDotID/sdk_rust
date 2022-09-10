use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::proof_service::{Action, Platform};

#[derive(Serialize)]
pub struct Request {
    pub action: Action,
    pub platform: Platform,
    pub identity: String,
    /// 0xUNCOMPRESSED_PUBKEY_HEXSTRING
    pub public_key: String,
    pub extra: Option<RequestExtra>,
}

#[derive(Serialize)]
pub struct RequestExtra {
    #[serde(rename = "wallet_signature")]
    pub ethereum_wallet_signature: String,
}

#[derive(Deserialize)]
pub struct Response {
    pub post_content: HashMap<String, String>,
    pub sign_payload: String,
    pub uuid: String,
    pub created_at: String,
}
