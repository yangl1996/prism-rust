use crate::crypto::hash::{Hashable, H256};
use byteorder::{BigEndian, ByteOrder};
use ring::rand;
use ring::signature::KeyPair as _;
use ring::signature::{self, Ed25519KeyPair};
use untrusted;

/// An object that can be meaningfully signed and verified.
pub trait Signable {
    /// Sign the object using Ed25519 with the given key pair.
    fn sign(&self, keypair: &KeyPair) -> Signature;

    /// Verify the object against the given public key and the signature.
    fn verify(&self, public_key: &PubKey, signature: &Signature) -> bool;
}

/// An Ed25519 signature.
#[derive(Serialize, Deserialize, Hash, Clone, Default, PartialEq, Eq, Copy)]
pub struct Signature([u128; 4]); // big endian u512

impl std::convert::From<&[u8; 64]> for Signature {
    fn from(input: &[u8; 64]) -> Signature {
        let u1 = BigEndian::read_u128(&input[0..16]);
        let u2 = BigEndian::read_u128(&input[16..32]);
        let u3 = BigEndian::read_u128(&input[32..48]);
        let u4 = BigEndian::read_u128(&input[48..64]);
        return Signature([u1, u2, u3, u4]);
    }
}

impl std::convert::From<&Signature> for [u8; 64] {
    fn from(input: &Signature) -> [u8; 64] {
        let mut buffer: [u8; 64] = [0; 64];
        BigEndian::write_u128(&mut buffer[0..16], input.0[0]);
        BigEndian::write_u128(&mut buffer[16..32], input.0[1]);
        BigEndian::write_u128(&mut buffer[32..48], input.0[2]);
        BigEndian::write_u128(&mut buffer[48..64], input.0[3]);
        return buffer;
    }
}

impl std::convert::From<[u8; 64]> for Signature {
    fn from(input: [u8; 64]) -> Signature {
        let u1 = BigEndian::read_u128(&input[0..16]);
        let u2 = BigEndian::read_u128(&input[16..32]);
        let u3 = BigEndian::read_u128(&input[32..48]);
        let u4 = BigEndian::read_u128(&input[48..64]);
        return Signature([u1, u2, u3, u4]);
    }
}

impl std::convert::From<Signature> for [u8; 64] {
    fn from(input: Signature) -> [u8; 64] {
        let mut buffer: [u8; 64] = [0; 64];
        BigEndian::write_u128(&mut buffer[0..16], input.0[0]);
        BigEndian::write_u128(&mut buffer[16..32], input.0[1]);
        BigEndian::write_u128(&mut buffer[32..48], input.0[2]);
        BigEndian::write_u128(&mut buffer[48..64], input.0[3]);
        return buffer;
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let start = if let Some(precision) = f.precision() {
            if precision >= 128 {
                0
            } else {
                64 - precision / 2
            }
        } else {
            0
        };
        let buffer: [u8; 64] = self.into();
        for byte_idx in start..64 {
            write!(f, "{:>02x}", &buffer[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let buffer: [u8; 64] = self.into();
        write!(
            f,
            "{:>02x}{:>02x}..{:>02x}{:>02x}",
            &buffer[0], &buffer[1], &buffer[62], &buffer[63]
        )
    }
}

/// An Ed25519 public key.
#[derive(Serialize, Deserialize, Hash, Clone, Default, PartialEq, Eq, Copy)]
pub struct PubKey([u128; 2]); // big endian u256.

impl std::convert::From<&[u8; 32]> for PubKey {
    fn from(input: &[u8; 32]) -> PubKey {
        let high = BigEndian::read_u128(&input[0..16]);
        let low = BigEndian::read_u128(&input[16..32]);
        return PubKey([high, low]);
    }
}

impl std::convert::From<&PubKey> for [u8; 32] {
    fn from(input: &PubKey) -> [u8; 32] {
        let mut buffer: [u8; 32] = [0; 32];
        BigEndian::write_u128(&mut buffer[0..16], input.0[0]);
        BigEndian::write_u128(&mut buffer[16..32], input.0[1]);
        return buffer;
    }
}

impl std::convert::From<[u8; 32]> for PubKey {
    fn from(input: [u8; 32]) -> PubKey {
        let high = BigEndian::read_u128(&input[0..16]);
        let low = BigEndian::read_u128(&input[16..32]);
        return PubKey([high, low]);
    }
}

impl std::convert::From<PubKey> for [u8; 32] {
    fn from(input: PubKey) -> [u8; 32] {
        let mut buffer: [u8; 32] = [0; 32];
        BigEndian::write_u128(&mut buffer[0..16], input.0[0]);
        BigEndian::write_u128(&mut buffer[16..32], input.0[1]);
        return buffer;
    }
}

impl std::fmt::Display for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let start = if let Some(precision) = f.precision() {
            if precision >= 64 {
                0
            } else {
                32 - precision / 2
            }
        } else {
            0
        };
        let buffer: [u8; 32] = self.into();
        for byte_idx in start..32 {
            write!(f, "{:>02x}", &buffer[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let buffer: [u8; 32] = self.into();
        write!(
            f,
            "{:>02x}{:>02x}..{:>02x}{:>02x}",
            &buffer[0], &buffer[1], &buffer[30], &buffer[31]
        )
    }
}

impl PubKey {
    // TODO: store public key as bytes to avoid the conversion
    // TODO: consider returning an Result<(), bool> to handle error
    /// Verify the message against the public key and the given signature.
    pub fn verify(&self, msg: &[u8], sig: &Signature) -> bool {
        let msg = untrusted::Input::from(msg);
        let pubkey_raw_bytes: [u8; 32] = self.into();
        let pubkey = untrusted::Input::from(&pubkey_raw_bytes);
        let signature_raw_bytes: [u8; 64] = sig.into();
        let signature = untrusted::Input::from(&signature_raw_bytes);
        let result = signature::verify(&signature::ED25519, pubkey, msg, signature);
        match result {
            Ok(()) => return true,
            _ => return false,
        }
    }
}

impl Hashable for PubKey {
    fn hash(&self) -> H256 {
        return ring::digest::digest(&ring::digest::SHA256, &bincode::serialize(&self).unwrap())
            .into();
    }
}

/// An Ed25519 key pair.
pub struct KeyPair {
    /// The underlying key pair readable by the crypto library.
    ring_keypair: Ed25519KeyPair,
    /// The key pair in PKCS8 format.
    pub pkcs8_bytes: Vec<u8>,
}

impl KeyPair {
    /// Generate a random key pair.
    pub fn random() -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair =
            Ed25519KeyPair::from_pkcs8(untrusted::Input::from(pkcs8_bytes.as_ref())).unwrap();
        return Self {
            ring_keypair: key_pair,
            pkcs8_bytes: pkcs8_bytes.as_ref().to_vec(),
        };
    }

    /// Get the public key of this key pair.
    pub fn public_key(&self) -> PubKey {
        let mut raw_pubkey: [u8; 32] = [0; 32];
        raw_pubkey[0..32].copy_from_slice(self.ring_keypair.public_key().as_ref());
        return raw_pubkey.into();
    }

    /// Sign the given message using this key pair.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        let mut raw_sig: [u8; 64] = [0; 64];
        raw_sig[0..64].copy_from_slice(self.ring_keypair.sign(msg).as_ref());
        return raw_sig.into();
    }

    /// Generate a key pair from pkcs8
    pub fn from_pkcs8(pkcs8_bytes: Vec<u8>) -> Self {
        let key_pair =
            Ed25519KeyPair::from_pkcs8(untrusted::Input::from(pkcs8_bytes.as_ref())).unwrap();
        return Self {
            ring_keypair: key_pair,
            pkcs8_bytes,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify() {
        let keypair = KeyPair::random();
        let message: [u8; 5] = [0, 1, 2, 3, 4];
        let public_key = keypair.public_key();
        let signature = keypair.sign(&message);
        assert!(public_key.verify(&message, &signature));
        let message_2: [u8; 5] = [9, 1, 2, 3, 4];
        assert!(!public_key.verify(&message_2, &signature));
    }

    #[test]
    fn serde() {
        let keypair = KeyPair::random();
        let pkcs8 = keypair.pkcs8_bytes.clone();
        let de_keypair = KeyPair::from_pkcs8(pkcs8);
        assert_eq!(keypair.public_key(), de_keypair.public_key());
        let message: [u8; 5] = [0, 1, 2, 3, 4];
        assert_eq!(keypair.sign(&message), de_keypair.sign(&message));
    }
}
