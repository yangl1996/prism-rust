use crate::crypto::hash::{Hashable, H256};
// TODO: Add the address of the miner

/// The header of a block.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Copy)]
pub struct Header {
    /// Hash of the parent proposer block.
    pub parent: H256,
    /// Block creation time in UNIX format.
    pub timestamp: u128,
    /// Proof of work nonce.
    pub nonce: u32,
    /// Merkle root of the block content.
    pub content_merkle_root: H256,
    /// Extra content for debugging purposes.
    pub extra_content: [u8; 32],
    /// Mining difficulty of this block.
    pub difficulty: H256,
}

impl Header {
    /// Create a new block header.
    pub fn new(
        parent: H256,
        timestamp: u128,
        nonce: u32,
        content_merkle_root: H256,
        extra_content: [u8; 32],
        difficulty: H256,
    ) -> Self {
        Self {
            parent,
            timestamp,
            nonce,
            content_merkle_root,
            extra_content,
            difficulty,
        }
    }
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized = bincode::serialize(self).unwrap();
        let digest = ring::digest::digest(&ring::digest::SHA256, &serialized);
        return digest.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::{Hashable, H256};

    fn sample_header() -> Header {
        let parent: H256 =
            (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into();
        let timestamp: u128 = 7094730;
        let nonce: u32 = 839782;
        let content_merkle_root: H256 =
            (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into();
        let extra_content: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ];
        let difficulty: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            20, 10,
        ];
        let difficulty = (&difficulty).into();
        let header = Header::new(
            parent,
            timestamp,
            nonce,
            content_merkle_root,
            extra_content,
            difficulty,
        );
        return header;
    }

    fn sample_header_hash_should_be() -> H256 {
        let header_hash_should_be =
            (&hex!("1d6e0f5f11248d070c6b15d103ead72a8802279c600a60b40b38510b582aacbf")).into(); // Calculated on Mar 15, 2019
        return header_hash_should_be;
    }
    // The hash should match
    #[test]
    fn test_hash() {
        let header = sample_header();
        let header_hash_should_be = sample_header_hash_should_be();
        assert_eq!(header.hash(), header_hash_should_be);
    }

    // Any changes to the header should change the hash value.
    #[test]
    fn fake_parent() {
        let mut header = sample_header();
        let header_hash_should_be = sample_header_hash_should_be();
        let fake_parent: H256 =
            (&hex!("0102010201020102010201020102010291790343908920102010201020102454")).into();
        header.parent = fake_parent;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_timestamp() {
        let mut header = sample_header();
        let header_hash_should_be = sample_header_hash_should_be();
        let fake_timestamp: u128 = 73948732;
        header.timestamp = fake_timestamp;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_nonce() {
        let mut header = sample_header();
        let header_hash_should_be = sample_header_hash_should_be();
        let fake_nonce: u32 = 209830934;
        header.nonce = fake_nonce;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_content_root() {
        let mut header = sample_header();
        let header_hash_should_be = sample_header_hash_should_be();
        let fake_content_root: H256 =
            (&hex!("beefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef")).into();
        header.content_merkle_root = fake_content_root;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_extra_content() {
        let mut header = sample_header();
        let header_hash_should_be = sample_header_hash_should_be();
        let fake_extra_content: [u8; 32] = [
            1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ];
        header.extra_content = fake_extra_content;
        assert_ne!(header.hash(), header_hash_should_be);
    }

    #[test]
    fn fake_difficulty() {
        let mut header = sample_header();
        let header_hash_should_be = sample_header_hash_should_be();
        let fake_difficulty: [u8; 32] = [
            1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ];
        header.difficulty = (&fake_difficulty).into();
        assert_ne!(header.hash(), header_hash_should_be);
    }
}
