mod procedure;
mod types;
pub use self::types::Action;
pub use self::types::Platform;
pub use procedure::{ProcedureStatus, ProofProcedure};

use self::types::Avatar;
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
    /// Fetch records by given `platform` and `identity`.
    /// If `fetch_all == true`, fetch all records till pagination ends.
    /// If not, only fetch first page of the results.
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use nextid_sdk::proof_service::{Endpoint, Platform};
    /// let avatars = Endpoint::Staging.find_by(Platform::Twitter, "yeiwb", false).await.unwrap();
    /// # assert!(avatars.len() > 0)
    /// # }
    /// ```
    pub async fn find_by(
        &self,
        platform: Platform,
        identity: &str,
        fetch_all: bool,
    ) -> Result<Vec<Avatar>> {
        let mut result: Vec<Avatar> = vec![];
        let mut page: usize = 1;
        loop {
            let single_page = self
                .find_by_single_page(&platform.to_string(), identity, page)
                .await?;
            single_page.ids.into_iter().for_each(|avatar| {
                result.push(avatar.into());
            });
            if !fetch_all || (single_page.pagination.next == 0) {
                break;
            }
            page += 1;
        }

        Ok(result)
    }

    /// Find a record by given platform and identity.
    async fn find_by_single_page(
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
}
