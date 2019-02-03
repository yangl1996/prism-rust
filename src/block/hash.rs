extern crate ring;

pub trait Hashable {
    fn hash(&self) -> Hash;
}

#[derive(Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Hash(pub [u8; 32]); // big endian u256

impl Hashable for Hash {
    fn hash(&self) -> Hash {
        let digest = ring::digest::digest(&ring::digest::SHA256, &self.0);
        // TODO: wait for try_into to stablize
        //let raw_hash: [u8; 32] = digest.as_ref().try_into().unwrap();
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[0..32].clone_from_slice(digest.as_ref());
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
        let bigger_hash = Hash([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ]);
        let smaller_hash = Hash([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(bigger_hash > smaller_hash, true);

        let bigger_hash = Hash([
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        let smaller_hash = Hash([
            0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(bigger_hash > smaller_hash, true);

        let some_hash = Hash([
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        let same_hash = Hash([
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(some_hash >= same_hash, true);
        assert_eq!(some_hash <= same_hash, true);
        assert_eq!(some_hash == same_hash, true);
    }

    #[test]
    fn from_u8() {
        let source: [u8; 32] = [
            1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2,
            2, 2, 2,
        ];
        let should_be = Hash([
            1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2,
            2, 2, 2,
        ]);
        let result: Hash = Hash::from(source);
        assert_eq!(result, should_be);
    }

    #[test]
    fn into_u8() {
        let should_be = [
            1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2,
            2, 2, 2,
        ];
        let source = Hash([
            1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2,
            2, 2, 2,
        ]);
        let result: [u8; 32] = source.into();
        assert_eq!(result, should_be);
    }

    #[test]
    fn hash() {
        let hash = Hash([
            32, 23, 32, 23, 32, 23, 32, 23, 32, 23, 32, 23, 32, 23, 32, 23, 32, 23, 32, 23, 32, 23,
            32, 23, 32, 23, 32, 23, 32, 23, 32, 23,
        ]);
        let hashed_hash = hash.hash();
        let should_be = Hash([
            0xcd, 0x9b, 0x88, 0xd7, 0x31, 0x9c, 0xaa, 0xf1, 0x6b, 0xed, 0x3f, 0xd6, 0xd4, 0x88,
            0x02, 0x84, 0xe0, 0x28, 0x34, 0x14, 0xb0, 0xb4, 0x4c, 0x22, 0x97, 0x8f, 0x7d, 0xc2,
            0x2d, 0x74, 0x17, 0x13,
        ]);
        assert_eq!(hashed_hash, should_be);
    }
}
