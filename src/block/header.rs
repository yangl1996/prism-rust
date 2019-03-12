use crate::crypto::hash::{Hashable, H256};

// TODO: Add the address of the miner

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
// TODO: discuss. PartialEq and Default are removed for now
pub struct Header{
    /// Hash of the parent proposer block.
    pub parent_hash: H256,
    /// Block creation time.
    pub timestamp: u64,
    /// Proof of work nonce.
    pub nonce: u32,
    /// Merkle root of the block content.
    pub content_root: H256,
    /// Extra content for debugging purposes.
    pub extra_content: Vec<u32>,
    /// Mining difficulty
    pub difficulty: [u8; 32],
    // TODO: discuss. Hash is removed for now. Do we need to "cache" the hash of a header?
}

impl Header{
    /// Create a new block header
    pub fn new(parent_hash: H256, timestamp: u64, nonce: u32, content_root: H256,
               extra_content: Vec<u32>, difficulty: [u8; 32] ) -> Self{
        Self{ parent_hash, timestamp, nonce, content_root, extra_content, difficulty}
    }

    // TODO: discuss. Mining-related logic are removed for now. Removed functions: set_nonce,
    // compute_hash, check_difficulty. We will do this in the miner logic.

}

impl Hashable for Header{
    fn hash(&self) -> H256 {
        unimplemented!();
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!();
    }
}
