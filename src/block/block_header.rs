extern crate bincode;
extern crate ring;

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
        let should_be = hash::Hash(hex!(
            "29e6703a080f122e9ac455aedfbe9bd1974492df74f88ad970c07b824d4ea292"
        ));
        assert_eq!(hash == should_be, true);
    }
}
