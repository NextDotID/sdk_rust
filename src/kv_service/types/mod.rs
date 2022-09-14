pub(crate) mod raw;

use crate::{proof_service::Platform, util::crypto::Secp256k1KeyPair};
use serde::Deserialize;
use serde_json::Value;

pub struct KVAvatar {
    pub avatar: Secp256k1KeyPair,
    pub content: Value,
}

#[derive(Deserialize)]
pub struct KVSingleProof {
    pub platform: Platform,
    pub identity: String,
    pub content: serde_json::Value,
}
