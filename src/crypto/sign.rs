use crate::crypto::hash::{Hashable, H256};
use byteorder::{BigEndian, ByteOrder};
use ring::rand;
use ring::signature::KeyPair as KeyPairTrait;
use ring::signature::{self, Ed25519KeyPair};
use untrusted;

/// An Ed25519 signature.
#[derive(Serialize, Deserialize, Hash, Clone, Default, PartialEq, Eq)]
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

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let buffer: [u8; 64] = self.into();
        for byte_idx in 0..64 {
            write!(f, "{:>02x}", &buffer[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let buffer: [u8; 64] = self.into();
        for byte_idx in 0..64 {
            write!(f, "{:>02x}", &buffer[byte_idx])?;
        }
        Ok(())
    }
}

/// An Ed25519 public key.
#[derive(Serialize, Deserialize, Hash, Clone, Default, PartialEq, Eq)]
pub struct PubKey([u128; 2]); // big endian u256. TODO: Use Crypto

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

impl std::fmt::Display for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let buffer: [u8; 32] = self.into();
        for byte_idx in 0..32 {
            write!(f, "{:>02x}", &buffer[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let buffer: [u8; 32] = self.into();
        for byte_idx in 0..32 {
            write!(f, "{:>02x}", &buffer[byte_idx])?;
        }
        Ok(())
    }
}

impl PubKey {
    // TODO: store public key as bytes to avoid the conversion
    // TODO: consider returning an Result<(), some_type>
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

pub struct KeyPair {
    ring_keypair: Ed25519KeyPair,
    pkcs8_bytes: Vec<u8>,
}

impl KeyPair {
    // generate a random Ed25519 key pair and the corresponding PKCS8 string
    pub fn new() -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair =
            Ed25519KeyPair::from_pkcs8(untrusted::Input::from(pkcs8_bytes.as_ref())).unwrap();
        return Self {
            ring_keypair: key_pair,
            pkcs8_bytes: pkcs8_bytes.as_ref().to_vec(),
        };
    }

    // get the public key of the key pair
    pub fn public_key(&self) -> PubKey {
        let mut raw_pubkey: [u8; 32] = [0; 32];
        raw_pubkey[0..32].copy_from_slice(self.ring_keypair.public_key().as_ref());
        return (&raw_pubkey).into();
    }

    // sign the message using the key pair
    pub fn sign(&self, msg: &[u8]) -> Signature {
        let mut raw_sig: [u8; 64] = [0; 64];
        raw_sig[0..64].copy_from_slice(self.ring_keypair.sign(msg).as_ref());
        return (&raw_sig).into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify() {
        let keypair = KeyPair::new();
        let message: [u8; 5] = [0, 1, 2, 3, 4];
        let public_key = keypair.public_key();
        let signature = keypair.sign(&message);
        let result = public_key.verify(&message, &signature);
        assert!(result);
    }
}

impl Hashable for PubKey {
    fn hash(&self) -> H256 {
        return ring::digest::digest(&ring::digest::SHA256, &bincode::serialize(&self).unwrap())
            .into();
    }
}
