use super::KVSingleProof;
use serde::Deserialize;
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
