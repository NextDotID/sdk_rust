## Rust SDK for NextID

### Components supported

- [x] [ProofService](https://docs.next.id/proof-service/ps-intro)
- [ ] [KVService](https://docs.next.id/kv-service/kv-intro)

### Usage

#### ProofService

##### Find binding records by given platform and identity.
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

##### Submit a ProofChain modification to ProofService server.

Run `cargo run --example procedure` to play an interactive demo.

See [examples/procedure.rs](./examples/procedure.rs) for more info.


#### Misc

You may find many useful functions under [`nextid_sdk::util`](./src/util/mod.rs) namespaces.
