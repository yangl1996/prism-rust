extern crate ring;

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

#[cfg(test)]
mod tests {
    use super::Hashable;
    use super::H256;

    #[test]
    fn ordering() {
        let bigger_hash = H256(hex!(
            "0000000000000000000000000000000000000000000000000000000000000001"
        ));
        let smaller_hash = H256(hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(bigger_hash > smaller_hash, true);

        let bigger_hash = H256(hex!(
            "0001000000000000000000000000000000000000000000000000000000000001"
        ));
        let smaller_hash = H256(hex!(
            "0000010000000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(bigger_hash > smaller_hash, true);

        let some_hash = H256(hex!(
            "0001000000000000000000000000000000000000000000000000000000000000"
        ));
        let same_hash = H256(hex!(
            "0001000000000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(some_hash >= same_hash, true);
        assert_eq!(some_hash <= same_hash, true);
        assert_eq!(some_hash == same_hash, true);
    }

    #[test]
    fn from_u8() {
        let source = hex!("0101010102020202010101010202020201010101020202020101010102020202");
        let should_be = H256(hex!(
            "0101010102020202010101010202020201010101020202020101010102020202"
        ));
        let result: H256 = H256::from(source);
        assert_eq!(result, should_be);
    }

    #[test]
    fn into_u8() {
        let should_be = hex!("0101010102020202010101010202020201010101020202020101010102020202");
        let source = H256(hex!(
            "0101010102020202010101010202020201010101020202020101010102020202"
        ));
        let result: [u8; 32] = source.into();
        assert_eq!(result, should_be);
    }

    #[test]
    fn hash() {
        let hash = H256(hex!(
            "2017201720172017201720172017201720172017201720172017201720172017"
        ));
        let hashed_hash = hash.sha256();
        let should_be: [u8; 32] =
            hex!("cd9b88d7319caaf16bed3fd6d4880284e0283414b0b44c22978f7dc22d741713");
        let should_be = H256(should_be);
        assert_eq!(hashed_hash, should_be);
    }
}
