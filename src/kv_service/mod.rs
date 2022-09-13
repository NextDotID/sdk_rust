mod types;

use self::types::raw::QueryResponse;
use self::types::{KVAvatar, KVSingleProof};
use crate::proof_service::Platform;
use crate::types::Result;
use crate::util::crypto::Secp256k1KeyPair;
use crate::util::hex_encode;
use crate::util::http::request;
use http::Method;
use hyper::Body;
use std::borrow::Borrow;
use url::Url;

/// KVService endpoint
#[derive(Debug, Clone)]
pub enum Endpoint {
    /// NextID production server
    /// https://proof-service.next.id
    Production,
    /// NextID staging server
    /// https://proof-service.nextnext.id
    Staging,
    /// Custom server (with full URL to the root of the server)
    Custom(String),
}

impl Endpoint {
    /// Concat server API URL.
    fn uri<I, K, V>(&self, path: &str, query: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        use Endpoint::*;
        let base = match self {
            Production => format!("https://kv-service.next.id/{}", path),
            Staging => format!("https://kv-service.nextnext.id/{}", path),
            Custom(url) => format!("{}/{}", url, path),
        };
        Url::parse_with_params(&base, query).map_err(|e| e.into())
    }

    /// Get all KV records under an avatar.
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use nextid_sdk::kv_service::Endpoint;
    /// # use nextid_sdk::util::crypto::Secp256k1KeyPair;
    /// let avatar = Secp256k1KeyPair::from_pk_hex("0x047e55e1b78e873c6f7d585064b41cd2735000bacc0092fe947c11ab7742ed351fef59c4f5d558d14a031bb09e44877f9e61f89993f895eb8fa6cfaafe74f6f55c").unwrap();
    /// let result = Endpoint::Staging.find_by_avatar(&avatar).await.unwrap();
    /// assert!(result.len() > 0);
    /// # }
    /// ```
    pub async fn find_by_avatar(&self, avatar: &Secp256k1KeyPair) -> Result<Vec<KVSingleProof>> {
        let pubkey_compress_hex = format!("0x{}", hex_encode(&avatar.pk.serialize_compressed()));
        let uri = self.uri("v1/kv", &[("avatar", pubkey_compress_hex)])?;
        let response: QueryResponse = request(Method::GET, &uri, Body::empty()).await?;

        Ok(response.proofs)
    }

    /// Get all KV records under a given platform / identity pair.
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use nextid_sdk::kv_service::Endpoint;
    /// # use nextid_sdk::proof_service::Platform;
    /// let result = Endpoint::Staging.find_by_platform_identity(Platform::Twitter, "yeiwb").await.unwrap();
    /// assert!(result.len() > 0);
    /// # }
    /// ```
    pub async fn find_by_platform_identity(
        &self,
        platform: Platform,
        identity: &str,
    ) -> Result<Vec<KVAvatar>> {
        let uri = self.uri(
            "v1/kv/by_identity",
            &[
                ("platform", platform.to_string().as_str()),
                ("identity", identity),
            ],
        )?;
        let response: types::raw::QueryIdentityResponse =
            request(Method::GET, &uri, Body::empty()).await?;
        response
            .values
            .into_iter()
            .map(|resp| match Secp256k1KeyPair::from_pk_hex(&resp.avatar) {
                Ok(avatar) => Ok(KVAvatar {
                    avatar,
                    content: resp.content,
                }),
                Err(err) => Err(err),
            })
            .collect()
    }
}
