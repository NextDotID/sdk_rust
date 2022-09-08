pub(crate) mod query;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Platform {
    #[serde(rename = "github")]
    Github,
    #[serde(rename = "nextid")]
    NextID,
    #[serde(rename = "twitter")]
    Twitter,
    #[serde(rename = "keybase")]
    Keybase,
    #[serde(rename = "ethereum")]
    Ethereum,
    #[serde(rename = "discord")]
    Discord,
    #[serde(rename = "dotbit")]
    Das,
    #[serde(rename = "solana")]
    Solana,
}
