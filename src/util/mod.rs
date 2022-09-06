#[cfg(test)]
mod tests;

use sha3::{Keccak256, Digest};
use crate::types::Result;

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

/// Decode a hexstring (`[a-f0-9]+`) to byte vec.
/// # Examples
/// ```rust
/// # use nextid_sdk::util::hex_decode;
/// let hexstring = "01020304";
/// let expected: Vec<u8> = vec![1, 2, 3, 4];
/// assert_eq!(expected, hex_decode(hexstring).unwrap());
/// ```
pub fn hex_decode(hexstring: &str) -> Result<Vec<u8>> {
    hex::decode(hexstring).map_err(|e| e.into())
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
/// # Example
/// ```rust
/// # use nextid_sdk::crypto::hash_keccak256;
/// # use hex_literal::hex;
/// #
/// let result = hash_keccak256(&"Test123");
/// let expected: [u8; 32] = hex!("504AF7475B7341893F803C8EBABFBAEA60EAE7B6A42CB006960C3FDB14DCF8AD");
/// assert_eq!(result, expected);
/// ```
pub fn keccak256_hash(message: &str) -> [u8; 32] {
    let mut hasher = Keccak256::default();
    hasher.update(message);
    hasher.finalize().into()
}
