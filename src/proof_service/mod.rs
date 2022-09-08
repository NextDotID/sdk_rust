mod types;

use crate::{types::Result, util::http::request};
use http::Method;
use hyper::Body;
use std::borrow::Borrow;
use url::Url;

/// ProofService endpoint
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
            Production => format!("https://proof-service.next.id/{}", path),
            Staging => format!("https://proof-service.nextnext.id/{}", path),
            Custom(url) => format!("{}/{}", url, path),
        };
        Url::parse_with_params(&base, query).map_err(|e| e.into())
    }

    /// Find a record by given platform and identity.
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use nextid_sdk::proof_service::Endpoint;
    /// let staging = Endpoint::Staging;
    /// let response = staging.find_by("twitter", "yeiwb", 1).await.unwrap();
    /// # assert!(response.pagination.total > 0)
    /// # }
    /// ```
    pub async fn find_by(
        &self,
        platform: &str,
        identity: &str,
        page: usize,
    ) -> Result<types::raw::query::Response> {
        let uri = self.uri(
            "v1/proof",
            &[
                ("platform", platform),
                ("identity", identity),
                ("page", &page.to_string()),
            ],
        )?;

        request(Method::GET, &uri, Body::empty()).await
    }
}
