extern crate bincode;

use super::hash;

#[derive(Serialize, Deserialize)]
pub struct BlockHeader {
    pub voter_hash: hash::Hash,
    pub proposal_hash: hash::Hash,
    pub transactions_hash: hash::Hash,
    pub nonce: u32,
}

impl std::fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{\n")?;
        write!(f, "  voter hash: {}\n", self.voter_hash)?;
        write!(f, "  proposal hash: {}\n", self.proposal_hash)?;
        write!(f, "  transactions hash: {}\n", self.transactions_hash)?;
        write!(f, "  nonce: {}\n", self.nonce)?;
        write!(f, "}}")
    }
}

impl hash::Hashable for BlockHeader {
    fn hash(&self) -> hash::Hash {
        let serialized = bincode::serialize(&self).unwrap();
        let digest = ring::digest::digest(&ring::digest::SHA256, &serialized);
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[..32].clone_from_slice(digest.as_ref());
        return raw_hash.into();
    }
}

#[cfg(test)]
mod tests {
    use super::super::hash;
    use super::super::hash::Hashable;
    use super::BlockHeader;

    #[test]
    fn hash() {
        let block_header = BlockHeader {
            voter_hash: hash::Hash([1; 32]),
            proposal_hash: hash::Hash([2; 32]),
            transactions_hash: hash::Hash([3; 32]),
            nonce: 12345,
        };
        let hash = block_header.hash();
        let should_be = hash::Hash([
            0x29, 0xe6, 0x70, 0x3a, 0x08, 0x0f, 0x12, 0x2e, 0x9a, 0xc4, 0x55, 0xae, 0xdf, 0xbe,
            0x9b, 0xd1, 0x97, 0x44, 0x92, 0xdf, 0x74, 0xf8, 0x8a, 0xd9, 0x70, 0xc0, 0x7b, 0x82,
            0x4d, 0x4e, 0xa2, 0x92,
        ]);
        assert_eq!(hash == should_be, true);
    }
}
