## Rust SDK for NextID

### Components supported

- [ProofService](https://docs.next.id/proof-service/ps-intro)
- [KVService](https://docs.next.id/kv-service/kv-intro)

### Usage

#### ProofService

```rust
use sdk_rust::{proof_service::Endpoint, types::Result};

#[tokio::main]
async fn main() -> Result<()> {
  let ps = Endpoint::Production;
  // Or use your own ProofService instance:
  // let ps = Endpoint::Custom("https://my-proof-service.example.com".to_string());

  let avatars = ps.find_by("twitter", "yeiwb", 1).await?;

}
```
