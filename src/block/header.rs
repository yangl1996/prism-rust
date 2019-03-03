extern crate ring;
use super::crypto::hash::{Hashable, H256};
use serde::{Serialize, Deserialize};
use std::fmt;


// ToDo: Add the address of the miner
// ToDo: #[derive(Serialize, Deserialize, Eq, Debug, Clone)]
// ToDo: Encoder and decoder for the blockheader?
// ToDo: Create default header function ?

pub struct BlockHeader{
    ///  Parent Hash
    parent_hash: H256,
    /// Block time
    timestamp: u64,
    /// PoW nonce
    nonce: u32,
    /// Merkle root of the block content
    content_root: H256,
    /// Block content
    extra_content: Vec<u32>, //for debugging purpose
    /// Mining Difficulty
    difficulty: u64,
    /// Hash of the header
    hash: Option<H256>
}

impl BlockHeader{

    /// Create a new block header
    pub fn new(parent_hash: H256, timestamp: u64, nonce: u32, content_root: H256,
               extra_content: Vec<u32>, difficulty: u64 ) -> Self{
        BlockHeader { parent_hash, timestamp, nonce, content_root, extra_content, difficulty, hash: None }
    }

    pub fn parent_hash(&self) -> &H256 { &self.parent_hash }

    /// Get the timestamp field of the header.
    pub fn timestamp(&self) -> u64 { self.timestamp.clone() }

    /// Get the content root field of the header.
    pub fn content_root(&self) -> &H256 { &self.content_root }

    /// Get the hash  of extra data field of the header.
    pub fn extra_content(&self) -> &Vec<u32> { &self.extra_content }

    pub fn get_hash(&self) -> Option<H256> {self.hash}

    /// Replace the old nonce and recompute the hash
    pub fn set_nonce(&mut self, new_nonce: u32) {
        self.nonce = new_nonce;
        self.compute_hash();
    }

    /// Compute hash of the block. Part of PoW mining
    fn compute_hash(&mut self) {
        self.hash = Some(self.hash());
    }


    /// Check if the block satisfies the PoW difficulty
    fn check_difficulty(&mut self, difficulty_base: u32) -> bool {
        self.compute_hash();
        // ToDo: Returns true if the hash is less than than the difficulty_base
        return true;
    }

}

impl Hashable for BlockHeader{
    fn hash(&self) -> H256 {
        // ToDo: Serialize the object into a byte array
        // return the H256 of the byte array
        let x: [u8; 32] = [0; 32]; // Default (wrong) behaviour
        return H256(x);
    }
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{{\n")?;
        write!(f, "  Parent hash: {}\n", self.parent_hash)?;
        write!(f, "  Timestamp: {}\n", self.timestamp)?;
        write!(f, "  nonce: {}\n", self.nonce)?;
        write!(f, "  content root: {}\n", self.content_root)?;
//        write!(f, "  extra content : {}\n", self.extra_content)?; //ToDo:: To define display for vec?
        write!(f, "  difficulty: {}\n", self.difficulty)?;
//        write!(f, "  hash: {}\n", self.hash)?;
        write!(f, "}}")
        // ToDo: Display more fields
    }
}