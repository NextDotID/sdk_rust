use crate::types::{Error, Result};
use http::Response;
use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use serde::Deserialize;

pub async fn request<T>(method: Method, uri: &url::Url, request_body: Body) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let client = new_client();
    let mut response = client
        .request(
            Request::builder()
                .method(method)
                .uri(uri.to_string().parse::<http::Uri>().unwrap())
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header("User-Agent", "NextID-SDK-Rust/0.1.0")
                .body(request_body)
                .unwrap(),
        )
        .await?;
    if [StatusCode::OK, StatusCode::CREATED]
        .into_iter()
        .all(|status| status != response.status())
    {
        // TODO: Provide more error info here
        return Err(Error::ServerError(format!("Status: {}", response.status())));
    }

    parse_body(&mut response).await
}

fn new_client() -> Client<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    Client::builder().build::<_, Body>(https)
}

async fn parse_body<T>(resp: &mut Response<Body>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut body_bytes: Vec<u8> = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let mut chunk_bytes = chunk.unwrap().to_vec();
        body_bytes.append(&mut chunk_bytes);
    }
    let body = std::str::from_utf8(&body_bytes).unwrap();

    Ok(serde_json::from_str(body)?)
}
