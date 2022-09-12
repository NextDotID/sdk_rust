/// Crypto-related helper functions
pub mod crypto;
/// HTTP-related helper functions
pub(crate) mod http;
#[cfg(test)]
mod tests;

use crate::types::Result;
use chrono::NaiveDateTime;
use libsecp256k1::PublicKey;
use sha3::{Digest, Keccak256};

/// Encode a byte slice into hexstring (`[a-f0-9]+`).
/// # Examples
/// ```rust
/// # use nextid_sdk::util::hex_encode;
/// let data: Vec<u8> = vec![1, 2, 3, 4];
/// assert_eq!("01020304", hex_encode(&data));
/// ```
#[allow(unused)]
pub fn hex_encode<T>(byte_slice: &T) -> String
where
    T: AsRef<[u8]>,
{
    hex::encode(byte_slice)
}

/// Decode a hexstring (`[a-f0-9]+`, with or without `0x`) to byte vec.
/// # Examples
/// ```rust
/// # use nextid_sdk::util::hex_decode;
/// let hexstring = "0x01020304";
/// let expected: Vec<u8> = vec![1, 2, 3, 4];
/// assert_eq!(expected, hex_decode(hexstring).unwrap());
/// ```
pub fn hex_decode(hexstring: &str) -> Result<Vec<u8>> {
    let hex: &str = if hexstring.starts_with("0x") {
        &hexstring[2..]
    } else {
        hexstring
    };

    hex::decode(hex).map_err(|e| e.into())
}

/// Encode a byte slice into Base64.
/// # Examples
/// ```rust
/// # use nextid_sdk::util::base64_encode;
/// let data: Vec<u8> = vec![1, 2, 3, 4];
/// let base64 = base64_encode(&data);
/// assert_eq!("AQIDBA==", base64);
/// ```
#[allow(unused)]
pub fn base64_encode<T>(byte_slice: &T) -> String
where
    T: AsRef<[u8]>,
{
    base64::encode(byte_slice)
}

/// Decode a base64 string to byte vec.
/// # Examples
/// ```rust
/// # use nextid_sdk::util::base64_decode;
/// let base64 = "AQIDBA==";
/// let expected: Vec<u8> = vec![1, 2, 3, 4];
/// assert_eq!(expected, base64_decode(base64).unwrap())
/// ```
#[allow(unused)]
pub fn base64_decode(base64_string: &str) -> Result<Vec<u8>> {
    base64::decode(base64_string).map_err(|e| e.into())
}

/// Keccak256(message)
/// # Examples
/// ```rust
/// # use nextid_sdk::crypto::hash_keccak256;
/// # use hex_literal::hex;
/// #
/// let result = hash_keccak256(&"Test123");
/// let expected: [u8; 32] = hex!("504AF7475B7341893F803C8EBABFBAEA60EAE7B6A42CB006960C3FDB14DCF8AD");
/// assert_eq!(result, expected);
/// ```
pub fn keccak256_hash(message: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = Keccak256::default();
    hasher.update(message);
    hasher.finalize().into()
}

/// Parse `String` type, second-based timestamp to NaiveDateTime
/// Convert timestamp string (unit: second) to [NaiveDateTime](chrono::NaiveDateTime).
/// # Examples
/// ```rust
/// # use nextid_sdk::util::ts_string_to_naive;
/// let naive_dt = ts_string_to_naive("1662708890");
/// # assert_eq!(2022, naive_dt.year());
/// ```
pub fn ts_string_to_naive(timestamp: &str) -> Result<NaiveDateTime> {
    let timestamp: i64 = timestamp.parse()?;
    Ok(ts_to_naive(timestamp, 0))
}

/// Convert timestamp into NaiveDateTime struct.
pub fn ts_to_naive(seconds: i64, ms: u32) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(seconds, ms * 1000000)
}

/// Generate Ethereum address from a secp256k1 public key.
/// Examples
/// ```rust
/// # use nextid_sdk::util::crypto::Secp256k1KeyPair;
/// # use nextid_sdk::util::eth_address_from_public_key;
/// # use hex_literal::hex;
/// # let keypair = Secp256k1KeyPair::from_pk_hex("0x04ca9b4078fbf0bc6d68999d8dc770c6d8c919ee80885fa6beec00541e22c47761fe4f48a5ecfba9541250cd98ea5087050ee5715bd3afe2b8480460b1c7f724cb").unwrap();
/// let address = eth_address_from_public_key(&keypair.pk);
/// let expected = hex!("1F4F4108C8FA5D307520D407CD1C2B08ACC391B2");
/// assert_eq!(expected, address);
/// ```
pub fn eth_address_from_public_key(pubkey: &PublicKey) -> [u8; 20] {
    let mut address = [0u8; 20];
    let pubkey_hash = keccak256_hash(&pubkey.serialize()[1..]); // Omit 0x04 / 0x03 pubkey type indicator.
    address.copy_from_slice(&pubkey_hash[12..]);

    address
}
