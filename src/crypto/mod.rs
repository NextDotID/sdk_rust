use std::slice::SliceIndex;

use crate::{
    types::{Error, Result},
    util::{hex_decode, keccak256_hash},
};
use libsecp256k1::{Message, PublicKey, SecretKey, Signature, RecoveryId};

/// secp256k1 public / secret key pair in a struct.
pub struct Secp256k1KeyPair {
    /// Public key
    pub pk: PublicKey,
    /// Secret key. May be missing in verifying signature scenario.
    sk: Option<SecretKey>,
}

impl Secp256k1KeyPair {
    /// Generate a keypair.
    /// # Examples
    /// ```rust
    /// # use nextid_sdk::crypto::Secp256k1KeyPair;
    /// let mut rng = rand::rngs::OsRng;
    /// let keypair = Secp256k1KeyPair::generate(&mut rng);
    /// # assert!(keypair.sk.is_some())
    /// ```
    pub fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        let sk = SecretKey::random(rng);
        let pk = PublicKey::from_secret_key(&sk);
        Self { pk, sk: Some(sk) }
    }

    /// Parse full or compressed pubkey from hexstring.
    /// Both `0x...` and raw hexstring are supported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use nextid_sdk::crypto::Secp256k1KeyPair;
    /// # use hex_literal::hex;
    /// # let pk_hex = "0x04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575";
    /// let pair = Secp256k1KeyPair::from_pk_hex(pk_hex).unwrap();
    /// # assert_eq!(hex!("04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575"), pair.pk.serialize());
    /// ```
    pub fn from_pk_hex(pk_hex: &str) -> Result<Self> {
        let hex = if pk_hex.starts_with("0x") {
            &pk_hex[2..]
        } else {
            pk_hex
        };
        let pk_bytes = hex_decode(hex)?;
        Self::from_pk_vec(&pk_bytes)
    }

    /// Parse full or compressed pubkey from a `Vec<u8>`.
    /// Notice that length of this `vec` should be `65` (uncompressed) or `33` (compressed), otherwise `Err` will be returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use nextid_sdk::crypto::Secp256k1KeyPair;
    /// # use hex_literal::hex;
    /// # let pk: Vec<u8> = hex!("04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575").into();
    /// let pair = Secp256k1KeyPair::from_pk_vec(&pk).unwrap();
    /// # assert_eq!(hex!("04c7cacde73af939c35d527b34e0556ea84bab27e6c0ed7c6c59be70f6d2db59c206b23529977117dc8a5d61fa848f94950422b79d1c142bcf623862e49f9e6575"), pair.pk.serialize());
    /// ```
    pub fn from_pk_vec(pk_vec: &Vec<u8>) -> Result<Self> {
        // `None` will try both 65- and 33-bytes version  vvvv
        let pk = PublicKey::parse_slice(pk_vec.as_slice(), None)?;

        Ok(Self { pk, sk: None })
    }

    /// `web3.eth.personal.sign`
    /// # Examples
    ///
    /// ```rust
    /// # use nextid_sdk::crypto::Secp256k1KeyPair;
    /// # use hex_literal::hex;
    /// # use libsecp256k1::{SecretKey, PublicKey};
    /// #
    /// let sign_payload = "Test123!";
    /// # let secret_key = SecretKey::parse(&hex!("b5466835b2228927d8dc1194cf8e6f52ba4b4cdb49cc954f31565d0c30fd44c8")).unwrap();
    /// # let expected = hex!("bc14fed2a5ae2c5c7e793f2a45f4f9aad84c7caa56139ee4a802806c5bb1a9cf4baa0e2df71bf3d0a943fbfb177afc1bd9c17995a6f409928548f3318d3f9b6300");
    /// # let keypair = Secp256k1KeyPair {
    /// #     pk: PublicKey::from_secret_key(&secret_key),
    /// #     sk: Some(secret_key),
    /// # };
    /// let result = keypair.personal_sign(sign_payload).unwrap();
    /// # assert_eq!(expected, result.as_slice())
    /// ```
    pub fn personal_sign(&self, message: &str) -> Result<Vec<u8>> {
        let personal_message =
            format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        self.hashed_sign(&personal_message)
    }

    /// Signs `keccak256(message)`.
    /// Returns raw signature (r + s + v, 65-bytes).
    pub fn hashed_sign(&self, message: &str) -> Result<Vec<u8>> {
        if self.sk.is_none() {
            return Err(Error::Secp256k1Error(libsecp256k1::Error::InvalidSecretKey));
        }

        let hashed_message = keccak256_hash(message);

        let (signature, recovery_id) =
            libsecp256k1::sign(&Message::parse(&hashed_message), &self.sk.unwrap());

        let mut result: Vec<u8> = vec![];
        result.extend_from_slice(&signature.r.b32());
        result.extend_from_slice(&signature.s.b32());
        result.extend_from_slice(&[recovery_id.serialize()]);
        if result.len() != 65 {
            return Err(Error::Secp256k1Error(
                libsecp256k1::Error::InvalidInputLength,
            ));
        }
        Ok(result)
    }

    /// Recover pubkey from an `web3.eth.personal.sign` signature with given plaintext message.
    /// # Examples
    ///
    /// ```rust
    /// # use nextid_sdk::{crypto::Secp256k1KeyPair, util::base64_decode};
    /// # use hex_literal::hex;
    /// # use libsecp256k1::{SecretKey, PublicKey, verify};
    /// #
    /// # let secret_key = SecretKey::parse(&hex!("b5466835b2228927d8dc1194cf8e6f52ba4b4cdb49cc954f31565d0c30fd44c8")).unwrap();
    /// # let public_key = PublicKey::from_secret_key(&secret_key);
    /// let sign_payload = "Test123!";
    /// # let keypair = Secp256k1KeyPair {
    /// #   pk,
    /// #   sk: Some(secret_key),
    /// # };
    /// # let signature = keypair.personal_sign(sign_payload).unwrap();
    /// # let sig = base64_decode(signature).unwrap();
    /// # println!("{:?}", sig);
    ///
    /// let recovered_keypair = Secp256k1KeyPair::recover_from_personal_signature(&sig, &sign_payload).unwrap();
    /// assert_eq!(recovered_keypair.pk, public_key);
    /// ```
    pub fn recover_from_personal_signature(
        sig_r_s_recovery: &Vec<u8>,
        plain_payload: &str,
    ) -> Result<Self> {
        let personal_payload = format!(
            "\x19Ethereum Signed Message:\n{}{}",
            plain_payload.len(),
            plain_payload
        );
        let digest = keccak256_hash(&personal_payload);

        let mut recovery_id = sig_r_s_recovery
            .get(64)
            .ok_or_else(|| Error::Secp256k1Error(libsecp256k1::Error::InvalidInputLength))?
            .clone();

        if recovery_id == 27 || recovery_id == 28 {
            recovery_id -= 27;
        }
        if recovery_id != 0 && recovery_id != 1 {
            return Err(Error::Secp256k1Error(libsecp256k1::Error::InvalidSignature));
        }

        let signature = Signature::parse_standard_slice(&sig_r_s_recovery.as_slice()[..64])?;
        let pk = libsecp256k1::recover(
            &Message::parse(&digest),
            &signature,
            &RecoveryId::parse(recovery_id).unwrap(),
        )?;

        Ok(Self {
            pk,
            sk: None,
        })
    }
}
