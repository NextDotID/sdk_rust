use nextid_sdk::{
    kv_service::{Endpoint, KVProcedure},
    proof_service::{Action, Platform},
    types::Result,
    util::{base64_decode, crypto::Secp256k1KeyPair, hex_encode},
};
use serde_json::json;

fn gets() -> Option<String> {
    let mut temp = String::new();
    std::io::stdin()
        .read_line(&mut temp)
        .expect("Failed to read line");
    if temp.trim().len() == 0 {
        None
    } else {
        Some(temp.trim().to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Which twitter username you want to set KV?");
    let twitter_username = gets().expect("Twitter username must be provided.");

    println!("OK. Tell me your avatar public key:");
    let avatar: Secp256k1KeyPair;
    match gets() {
        None => {
            println!("Seems like you don't have an avatar yet. Let me generate one for you:");
            let mut rng = rand::rngs::OsRng;
            avatar = Secp256k1KeyPair::generate(&mut rng);
            println!(
                "Secret key: 0x{}",
                hex_encode(&avatar.sk.as_ref().unwrap().serialize())
            );
        }
        Some(pubkey_hex) => {
            avatar = Secp256k1KeyPair::from_pk_hex(&pubkey_hex)?;
        }
    }
    let avatar_pubkey = format!("0x{}", hex_encode(&avatar.pk.serialize_compressed()))
        .trim()
        .to_string();
    println!("Public key: {}", avatar_pubkey);

    let mut procedure = KVProcedure::new(
        Endpoint::Staging,
        Action::Create,
        avatar,
        Platform::Twitter,
        &twitter_username,
        json!({
            "this": {
                "is": "a"
            },
            "testcase": ["to", "do", "json", "patches"],
            "delete_this_key": null
        }),
    );
    procedure.get_payload().await?;

    println!("First, make sure this twitter-avatar pair has binding record on ProofService staging server.");
    println!("Ask user to sign this using their avatar secret key using web3.eth.personal.sign() method:\n\n{}\n\n", procedure.sign_payload.as_ref().unwrap());
    println!("Done? Base64 this signature and paste it here:\n");
    let base64_sig = gets().expect("Signature must be provided.");
    let sig = base64_decode(&base64_sig)?;
    procedure.submit(sig).await?;
    println!("Done.");

    Ok(())
}
