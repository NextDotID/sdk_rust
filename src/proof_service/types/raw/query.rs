use super::Platform;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Pagination {
    pub total: u64,
    pub per: usize,
    pub current: usize,
    pub next: usize,
}

#[derive(Serialize)]
pub struct Request {
    pub platform: Platform,
    pub identity: Vec<String>,
    pub page: Option<usize>,
}

#[derive(Deserialize)]
pub struct Response {
    pub pagination: Pagination,
    pub ids: Vec<ResponseSingleAvatar>,
}

#[derive(Deserialize)]
pub struct ResponseSingleAvatar {
    pub avatar: String,
    pub last_arweave_id: String,
    pub proofs: Vec<ResponseSingleProof>,
}

#[derive(Deserialize)]
pub struct ResponseSingleProof {
    pub platform: Platform,
    pub identity: String,
    pub created_at: String,
    pub last_checked_at: String,
    pub is_valid: bool,
    pub invalid_reason: String,
}
