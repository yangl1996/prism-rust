/// An object that can be meaningfully hashed.
pub trait Hashable {
    /// Hashes the object using SHA256.
        fn hash(&self) -> H256;
}

/// A SHA256 hash
#[derive(Eq, Serialize, Deserialize, Clone, Debug, Hash, Default, Copy)]
pub struct H256(pub [u8; 32]); // big endian u256


impl Hashable for H256 {
    fn hash(&self) -> H256 {
        return ring::digest::digest(&ring::digest::SHA256, &self.0).into();
    }
}

impl std::convert::From<[u8; 32]> for H256 {
    fn from(input: [u8; 32]) -> H256 {
        return H256(input);
    }
}

impl std::convert::From<H256> for [u8; 32] {
    fn from(input: H256) -> [u8; 32] {
        return input.0;
    }
}

impl std::convert::From<ring::digest::Digest> for H256 {
    fn from(input: ring::digest::Digest) -> H256 {
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[0..32].copy_from_slice(input.as_ref());
        return raw_hash.into();
    }
}

impl Ord for H256 {
    fn cmp(&self, other: &H256) -> std::cmp::Ordering {
        for byte_idx in 0..31 {
            let res = self.0[byte_idx].cmp(&other.0[byte_idx]);
            match res {
                std::cmp::Ordering::Equal => {
                    continue;
                }
                _ => {
                    return res;
                }
            }
        }
        return self.0[31].cmp(&other.0[31]);
    }
}

impl PartialOrd for H256 {
    fn partial_cmp(&self, other: &H256) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for H256 {
    fn eq(&self, other: &H256) -> bool {
        for byte_idx in 0..32 {
            if self.0[byte_idx] != other.0[byte_idx] {
                return false;
            }
        }
        return true;
    }
}

impl std::fmt::Display for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..32 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}
