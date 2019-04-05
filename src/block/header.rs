use crate::crypto::hash::{Hashable, H256};
// TODO: Add the address of the miner

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Copy)]
pub struct Header{
    /// Hash of the parent proposer block. Every block has a parent proposer block.
    pub parent_hash: H256,
    /// Block creation time.
    pub timestamp: u64,
    /// Proof of work nonce.
    pub nonce: u32,
    /// Merkle root of the block content.
    pub content_root: H256,
    /// Extra content for debugging purposes.
    pub extra_content: [u8; 32],
    /// Mining difficulty
    pub difficulty: [u8; 32],
}

impl Header{
    /// Create a new block header
    pub fn new(parent_hash: H256, timestamp: u64, nonce: u32, content_root: H256,
               extra_content: [u8; 32], difficulty: [u8; 32] ) -> Self{
        Self {parent_hash, timestamp, nonce, content_root, extra_content, difficulty}
    }
}


impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized = bincode::serialize(self).unwrap();
        let digest = ring::digest::digest(&ring::digest::SHA256, &serialized);
        return digest.into();
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "parent_hash: {}; timestamp: {}; self_hash: {}",
               self.parent_hash, self.timestamp, self.hash())?; // Ignoring status for now
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::hash::{Hashable, H256};
    use super::super::test_util;


    ///The hash should match
    #[test]
    fn test_hash() {
        let mut header = test_util::sample_header();
        let header_hash_should_be = test_util::sample_header_hash_should_be();
        assert_eq!(header.hash(), header_hash_should_be);
    }

    /// Any changes to the header should change the hash value.
    #[test]
    fn fake_parent(){
        let mut header = test_util::sample_header();
        let header_hash_should_be = test_util::sample_header_hash_should_be();
        let fake_parent_hash: H256 = (&hex!(
            "0102010201020102010201020102010291790343908920102010201020102454" )).into();
        header.parent_hash = fake_parent_hash;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_timestamp(){
        let mut header = test_util::sample_header();
        let header_hash_should_be = test_util::sample_header_hash_should_be();
        let fake_timestamp: u64 = 73948732;
        header.timestamp = fake_timestamp;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_nonce(){
        let mut header = test_util::sample_header();
        let header_hash_should_be = test_util::sample_header_hash_should_be();
        let fake_nonce: u32 = 209830934;
        header.nonce = fake_nonce;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_content_root(){
        let mut header = test_util::sample_header();
        let header_hash_should_be = test_util::sample_header_hash_should_be();
        let fake_content_root: H256 = (&hex!(
            "beefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef" )).into();
        header.content_root = fake_content_root;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_extra_content(){
        let mut header = test_util::sample_header();
        let header_hash_should_be = test_util::sample_header_hash_should_be();
        let fake_extra_content: [u8; 32] = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, ];
        header.extra_content = fake_extra_content;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_difficulty(){
        let mut header = test_util::sample_header();
        let header_hash_should_be = test_util::sample_header_hash_should_be();
        let fake_difficulty: [u8; 32] = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, ];
        header.difficulty = fake_difficulty;
        assert_ne!(header.hash(), header_hash_should_be);
    }

}