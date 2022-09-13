use crate::proof_service::Platform;

use super::KVSingleProof;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct QueryResponse {
    pub avatar: String,
    pub proofs: Vec<KVSingleProof>,
}

#[derive(Deserialize)]
pub struct QueryIdentityResponse {
    pub values: Vec<QueryIdentitySingleResponse>,
}

#[derive(Deserialize)]
pub struct QueryIdentitySingleResponse {
    pub avatar: String,
    pub content: Value,
}

#[derive(Serialize)]
pub struct PayloadRequest<'a> {
    pub avatar: &'a str,
    pub platform: &'a Platform,
    pub identity: &'a str,
    pub patch: &'a Value,
}

#[derive(Deserialize)]
pub struct PayloadResponse {
    pub uuid: String,
    pub sign_payload: String,
    pub created_at: i64,
}
