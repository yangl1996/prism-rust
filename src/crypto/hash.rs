use std::convert::TryInto;

/// An object that can be meaningfully hashed.
pub trait Hashable {
    /// Hash the object using SHA256.
    fn hash(&self) -> H256;
}

/// A SHA256 hash.
#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, Default, Copy)]
pub struct H256([u8; 32]); // big endian u256

impl std::fmt::Display for H256 {
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
        for byte_idx in start..32 {
            write!(f, "{:>02x}", &self.0[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:>02x}{:>02x}..{:>02x}{:>02x}",
            &self.0[0], &self.0[1], &self.0[30], &self.0[31]
        )
    }
}

impl Hashable for H256 {
    fn hash(&self) -> H256 {
        ring::digest::digest(&ring::digest::SHA256, &self.0).into()
    }
}

impl std::convert::AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::convert::From<&[u8; 32]> for H256 {
    fn from(input: &[u8; 32]) -> H256 {
        let mut buffer: [u8; 32] = [0; 32];
        buffer[..].copy_from_slice(input);
        H256(buffer)
    }
}

impl std::convert::From<&H256> for [u8; 32] {
    fn from(input: &H256) -> [u8; 32] {
        let mut buffer: [u8; 32] = [0; 32];
        buffer[..].copy_from_slice(&input.0);
        buffer
    }
}

impl std::convert::From<[u8; 32]> for H256 {
    fn from(input: [u8; 32]) -> H256 {
        H256(input)
    }
}

impl std::convert::From<H256> for [u8; 32] {
    fn from(input: H256) -> [u8; 32] {
        input.0
    }
}

impl std::convert::From<ring::digest::Digest> for H256 {
    fn from(input: ring::digest::Digest) -> H256 {
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[0..32].copy_from_slice(input.as_ref());
        H256(raw_hash)
    }
}

impl Ord for H256 {
    fn cmp(&self, other: &H256) -> std::cmp::Ordering {
        let self_higher = u128::from_be_bytes(self.0[0..16].try_into().unwrap());
        let self_lower = u128::from_be_bytes(self.0[16..32].try_into().unwrap());
        let other_higher = u128::from_be_bytes(other.0[0..16].try_into().unwrap());
        let other_lower = u128::from_be_bytes(other.0[16..32].try_into().unwrap());
        let higher = self_higher.cmp(&other_higher);
        match higher {
            std::cmp::Ordering::Equal => self_lower.cmp(&other_lower),
            _ => higher,
        }
    }
}

impl PartialOrd for H256 {
    fn partial_cmp(&self, other: &H256) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(any(test))]
pub mod tests {
    use super::Hashable;
    use super::H256;
    use rand::Rng;

    pub fn generate_random_hash() -> H256 {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen_range(0, 255) as u8).collect();
        let mut raw_bytes = [0; 32];
        raw_bytes.copy_from_slice(&random_bytes);
        (&raw_bytes).into()
    }

    #[test]
    fn ordering() {
        let bigger_hash: H256 =
            (&hex!("0000000000000000000000000000000000000000000000000000000000000001")).into();
        let smaller_hash: H256 =
            (&hex!("0000000000000000000000000000000000000000000000000000000000000000")).into();
        assert_eq!(bigger_hash > smaller_hash, true);

        let bigger_hash: H256 =
            (&hex!("0001000000000000000000000000000000000000000000000000000000000001")).into();
        let smaller_hash: H256 =
            (&hex!("0000010000000000000000000000000000000000000000000000000000000000")).into();
        assert_eq!(bigger_hash > smaller_hash, true);

        let some_hash: H256 =
            (&hex!("0001000000000000000000000000000000000000000000000000000000000000")).into();
        let same_hash: H256 =
            (&hex!("0001000000000000000000000000000000000000000000000000000000000000")).into();
        assert_eq!(some_hash >= same_hash, true);
        assert_eq!(some_hash <= same_hash, true);
        assert_eq!(some_hash == same_hash, true);
    }

    #[test]
    fn convert_u8() {
        let source = hex!("0101010102020202010101010202020201010101020202020101010102020202");
        let should_be: H256 =
            (&hex!("0101010102020202010101010202020201010101020202020101010102020202")).into();
        let result: H256 = H256::from(&source);
        assert_eq!(result, should_be);
    }

    #[test]
    fn asref_u8() {
        let source: H256 =
            (&hex!("0101010102020202010101010202020201010101020202020101010102020202")).into();
        assert_eq!(
            source.as_ref(),
            &hex!("0101010102020202010101010202020201010101020202020101010102020202")
        );
    }

    #[test]
    fn hash() {
        let hash: H256 =
            (&hex!("2017201720172017201720172017201720172017201720172017201720172017")).into();
        let hashed_hash = hash.hash();
        let should_be: [u8; 32] =
            hex!("cd9b88d7319caaf16bed3fd6d4880284e0283414b0b44c22978f7dc22d741713");
        let should_be: H256 = (&should_be).into();
        assert_eq!(hashed_hash, should_be);
    }
}
