/// An Ed25519 signature.
#[derive(Serialize, Deserialize)]
pub struct Signature(pub [u8; 64]);

impl std::convert::From<[u8; 64]> for Signature {
    fn from(input: [u8; 64]) -> Signature {
        return Signature(input);
    }
}

impl std::convert::From<Signature> for [u8; 64] {
    fn from(input: Signature) -> [u8; 64] {
        return input.0;
    }
}

impl std::convert::From<ring::signature::Signature> for Signature {
    fn from(input: ring::signature::Signature) -> Signature {
        let mut raw_sig: [u8; 64] = [0; 64];
        raw_sig[0..64].copy_from_slice(input.as_ref());
        return raw_sig.into();
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..64 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..64 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}

/// An Ed25519 pubkey.
#[derive(Serialize, Deserialize)]
pub struct PubKey(pub [u8; 32]);

impl std::convert::From<[u8; 32]> for PubKey {
    fn from(input: [u8; 32]) -> PubKey {
        return PubKey(input);
    }
}

impl std::convert::From<PubKey> for [u8; 32] {
    fn from(input: PubKey) -> [u8; 32] {
        return input.0;
    }
}

impl std::fmt::Display for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..32 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..32 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}
