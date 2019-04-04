use byteorder::{BigEndian, ByteOrder};

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

#[derive(Serialize, Deserialize, Hash, Clone, Default, PartialEq, Eq)]
pub struct SecKey([u128; 2]); // big endian u256.  TODO: Use Crypto



pub struct KeyPair {
    secret: SecKey,
    public: PubKey,
}