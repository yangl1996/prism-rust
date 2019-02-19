extern crate ring;

pub trait Hashable {
    fn hash(&self) -> Hash;
}

#[derive(Eq, Serialize, Deserialize, Clone, Debug, Hash)]
pub struct Hash(pub [u8; 32]); // big endian u256

impl Hashable for Hash {
    fn hash(&self) -> Hash {
        let digest = ring::digest::digest(&ring::digest::SHA256, &self.0);
        // TODO: wait for try_into to stablize
        //let raw_hash: [u8; 32] = digest.as_ref().try_into().unwrap();
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[0..32].copy_from_slice(digest.as_ref());
        return raw_hash.into();
    }
}

impl std::convert::From<[u8; 32]> for Hash {
    fn from(input: [u8; 32]) -> Hash {
        return Hash(input);
    }
}

impl std::convert::From<Hash> for [u8; 32] {
    fn from(input: Hash) -> [u8; 32] {
        return input.0;
    }
}

impl Ord for Hash {
    fn cmp(&self, other: &Hash) -> std::cmp::Ordering {
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

impl PartialOrd for Hash {
    fn partial_cmp(&self, other: &Hash) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Hash) -> bool {
        for byte_idx in 0..32 {
            if self.0[byte_idx] != other.0[byte_idx] {
                return false;
            }
        }
        return true;
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..32 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Hash;
    use super::Hashable;

    #[test]
    fn ordering() {
        let bigger_hash = Hash(hex!(
            "0000000000000000000000000000000000000000000000000000000000000001"
        ));
        let smaller_hash = Hash(hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(bigger_hash > smaller_hash, true);

        let bigger_hash = Hash(hex!(
            "0001000000000000000000000000000000000000000000000000000000000001"
        ));
        let smaller_hash = Hash(hex!(
            "0000010000000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(bigger_hash > smaller_hash, true);

        let some_hash = Hash(hex!(
            "0001000000000000000000000000000000000000000000000000000000000000"
        ));
        let same_hash = Hash(hex!(
            "0001000000000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(some_hash >= same_hash, true);
        assert_eq!(some_hash <= same_hash, true);
        assert_eq!(some_hash == same_hash, true);
    }

    #[test]
    fn from_u8() {
        let source = hex!("0101010102020202010101010202020201010101020202020101010102020202");
        let should_be = Hash(hex!(
            "0101010102020202010101010202020201010101020202020101010102020202"
        ));
        let result: Hash = Hash::from(source);
        assert_eq!(result, should_be);
    }

    #[test]
    fn into_u8() {
        let should_be = hex!("0101010102020202010101010202020201010101020202020101010102020202");
        let source = Hash(hex!(
            "0101010102020202010101010202020201010101020202020101010102020202"
        ));
        let result: [u8; 32] = source.into();
        assert_eq!(result, should_be);
    }

    #[test]
    fn hash() {
        let hash = Hash(hex!(
            "2017201720172017201720172017201720172017201720172017201720172017"
        ));
        let hashed_hash = hash.hash();
        let should_be: [u8; 32] =
            hex!("cd9b88d7319caaf16bed3fd6d4880284e0283414b0b44c22978f7dc22d741713");
        let should_be = Hash(should_be);
        assert_eq!(hashed_hash, should_be);
    }
}
