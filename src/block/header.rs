use crate::crypto::hash::{Hashable, H256};

// TODO: Add the address of the miner

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
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
    pub extra_content: [u8; 32],
    /// Mining difficulty
    pub difficulty: [u8; 32],
}

impl Header{
    /// Create a new block header
    pub fn new(parent_hash: H256, timestamp: u64, nonce: u32, content_root: H256,
               extra_content: [u8; 32], difficulty: [u8; 32] ) -> Self{
        Self{ parent_hash, timestamp, nonce, content_root, extra_content, difficulty}
    }
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

#[cfg(test)]
mod tests {
    use crate::crypto::hash::{Hashable, H256};
    use super::*;

    #[test]
    fn new_header()
    {
        let parent_hash :H256 = (&hex!(
            "0102010201020102010201020102010201020102010201020102010201020102" )).into();
        let timestamp :u64 = 7094730;
        let nonce: u32 = 839782;
        let content_root :H256 = (&hex!(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef" )).into();
        let extra_content :[u8; 32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,];
        let difficulty :[u8; 32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,20,10,];
        let header = Header::new(parent_hash, timestamp, nonce, content_root, extra_content, difficulty);
    }
}