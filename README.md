## Rust SDK for NextID

### Components supported

- [x] [ProofService](https://docs.next.id/proof-service/ps-intro)
- [x] [KVService](https://docs.next.id/kv-service/kv-intro)

### Usage

#### ProofService

##### Find binding records by given platform and identity.

```rust
use nextid_sdk::{proof_service::Endpoint, types::Result};

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

See [examples/proof_procedure.rs](./examples/proof_procedure.rs) for more info.

#### KVService

##### Find KV by given avatar

```rust
use nextid_sdk::{
  types::Result,
  kv_service::Endpoint,
  util::crypto::Secp256k1KeyPair
};

#[tokio::main]
async fn main() -> Result<()> {

  let avatar = Secp256k1KeyPair::from_pk_hex("0x047e55e1b78e873c6f7d585064b41cd2735000bacc0092fe947c11ab7742ed351fef59c4f5d558d14a031bb09e44877f9e61f89993f895eb8fa6cfaafe74f6f55c");

  let result = Endpoint::Staging.find_by_avatar(&avatar).await?;

  Ok(())
}
```

##### Find KV by given `platform` / `identity` pair

```rust
use nextid_sdk::{
  types::Result,
  kv_service::Endpoint,
  proof_service::Platform,
};

#[tokio::main]
async fn main() -> Result<()> {

  let result = Endpoint::Staging.find_by_platform_identity(Platform::Twitter, "yeiwb").await?;

  Ok(())
}
```

##### Submit a KV modification to remote KVService server.

See [examples/kv_procedure.rs](./examples/kv_procedure.rs)

#### Toolkits

You may find many useful functions under [`nextid_sdk::util`](./src/util/mod.rs) namespaces.
