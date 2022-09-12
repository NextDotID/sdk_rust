pub(crate) mod raw;

use self::raw::query::{AvatarWithProof, SingleProof};
use crate::util::{hex_decode, ts_string_to_naive};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// All actios available when modifying ProofChain.
#[derive(Serialize, Deserialize, Clone, Copy, Display, EnumString, Eq, PartialEq)]
pub enum Action {
    #[serde(rename = "create")]
    #[strum(serialize = "create")]
    Create,
    #[serde(rename = "delete")]
    #[strum(serialize = "delete")]
    Delete,
}

/// All platforms supported by ProofService
#[derive(Serialize, Deserialize, Clone, Copy, Display, EnumString, Eq, PartialEq)]
pub enum Platform {
    #[serde(rename = "github")]
    #[strum(serialize = "github")]
    Github,
    #[serde(rename = "nextid")]
    #[strum(serialize = "nextid")]
    NextID,
    #[serde(rename = "twitter")]
    #[strum(serialize = "twitter")]
    Twitter,
    #[serde(rename = "keybase")]
    #[strum(serialize = "keybase")]
    Keybase,
    #[serde(rename = "ethereum")]
    #[strum(serialize = "ethereum")]
    Ethereum,
    #[serde(rename = "discord")]
    #[strum(serialize = "discord")]
    Discord,
    #[serde(rename = "dotbit")]
    #[strum(serialize = "dotbit")]
    Das,
    #[serde(rename = "solana")]
    #[strum(serialize = "solana")]
    Solana,
}

/// Avatar record by query.
#[derive(Clone)]
pub struct Avatar {
    /// Avatar public key (secp256k1 public key, uncompressed, raw bytes).
    pub avatar: Vec<u8>,
    /// Arweave object ID for last ProofRecord record under this `avatar`.
    pub last_arweave_id: String,
    /// Current proofs available under this `avatar`.
    pub proofs: Vec<Proof>,
}

impl From<AvatarWithProof> for Avatar {
    fn from(raw_avatar: AvatarWithProof) -> Self {
        Self {
            avatar: hex_decode(&raw_avatar.avatar).expect("Error when decoding avatar hexstring"),
            last_arweave_id: raw_avatar.last_arweave_id,
            proofs: raw_avatar.proofs.into_iter().map(|p| p.into()).collect(),
        }
    }
}

/// Single proof record.
#[derive(Clone)]
pub struct Proof {
    /// Platform supported by ProofService
    pub platform: Platform,
    /// Identity on target `platform`
    pub identity: String,
    /// Creation datetime of this record.
    pub created_at: NaiveDateTime,
    /// When did ProofService checked this proof record.
    pub last_checked_at: NaiveDateTime,
    /// Last check status of this proof record.
    pub is_valid: bool,
    /// If `is_valid == false`, why?
    pub invalid_reason: Option<String>,
}

impl From<SingleProof> for Proof {
    fn from(raw_proof: SingleProof) -> Self {
        Self {
            platform: raw_proof.platform,
            identity: raw_proof.identity,
            created_at: ts_string_to_naive(&raw_proof.created_at)
                .expect("Error when parsing created_at"),
            last_checked_at: ts_string_to_naive(&raw_proof.created_at)
                .expect("Error when parsing last_checked_at"),
            is_valid: raw_proof.is_valid,
            invalid_reason: if raw_proof.invalid_reason.len() == 0 {
                None
            } else {
                Some(raw_proof.invalid_reason.clone())
            },
        }
    }
}
