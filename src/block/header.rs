/*
The blockheader and block struct is defined along with their initialization and other functions
*/

extern crate ring;
use super::crypto::hash::{Hashable, SHA256};
use serde::{Serialize, Deserialize};
use std::fmt;


pub struct PersistentModifier{
    parent_id: SHA256,
}

/// PersistentModifiers have parent modifiers defined all the way upto genesis blocks.
impl PersistentModifier{
    fn new(modifier_id:  SHA256) -> Self{
        PersistentModifier {parent_id: modifier_id}
    }
}

// ToDo: Add the address of the miner
// ToDo: #[derive(Serialize, Deserialize, Eq, Debug, Clone)]
// ToDo: Encoder and decoder for the blockheader?
// ToDo: Create default header function ?

pub struct BlockHeader{
    ///  Parent Hash
    parent: PersistentModifier,
    /// Block time
    timestamp: u64,
    /// PoW nonce
    nonce: u32,
    /// Merkle root of the block content
    content_root: SHA256,
    /// Block content
    extra_content: Vec<u32>, //for debugging purpose
    /// Mining Difficulty
    difficulty: u64,
    /// Hash of the header
    hash: Option<SHA256>
}

impl BlockHeader{

    /// Create a new block header
    pub fn new(parent_id: SHA256, timestamp: u64, nonce: u32, content_root: SHA256,
               extra_content: Vec<u32>, difficulty: u64 ) -> Self{
        BlockHeader { parent: PersistentModifier::new(parent_id),
                        timestamp, nonce, content_root, extra_content, difficulty, hash: None }
    }

    pub fn parent_hash(&self) -> &SHA256 { &self.parent.parent_id }

    /// Get the timestamp field of the header.
    pub fn timestamp(&self) -> u64 { self.timestamp }

    /// Get the content root field of the header.
    pub fn content_root(&self) -> &SHA256 { &self.content_root }

    /// Get the hash  of extra data field of the header.
    pub fn extra_content(&self) -> &Vec<u32> { &self.extra_content }

    pub fn hash(&self) -> Option<SHA256> {self.hash}

    /// Replace the old nonce and recompute the hash
    pub fn set_nonce(&mut self, new_nonce: u32) {
        self.nonce = new_nonce;
        self.compute_hash();
    }

    /// Compute hash of the block. Part of PoW mining
    fn compute_hash(&mut self) {
        self.hash = Some(self.sha256());
    }


    /// Check if the block satisfies the PoW difficulty
    fn check_difficulty(&mut self, difficulty_base: u32) -> bool {
        self.compute_hash();
        // ToDo: Returns true if the hash is less than than the difficulty_base
        return true;
    }

}

impl Hashable for BlockHeader{
    fn sha256(&self) -> SHA256 {
        // ToDo: Serialize the object into a byte array
        // return the SHA256 of the byte array
        let x: [u8; 32] = [0; 32]; // Default (wrong) behaviour
        return SHA256(x);
    }
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{{\n")?;
        write!(f, "  Parent Id: {}\n", self.parent.parent_id)?;
        write!(f, "  Timestamp: {}\n", self.timestamp)?;
        write!(f, "  nonce: {}\n", self.nonce)?;
        write!(f, "  content root: {}\n", self.content_root)?;
//        write!(f, "  extra content : {}\n", self.extra_content)?; ToDo:: To define default param
        write!(f, "}}")
        // ToDo: Display more fields
    }
}