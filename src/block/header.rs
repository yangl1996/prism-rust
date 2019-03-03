/*
The blockheader and block struct is defined along with their initialization and other functions
*/


extern crate ring;
use crate::hash::{self, Hashable, SHA256};
use serde::{Serialize, Deserialize};
use std::fmt;

/*
ToDo: Encoder and decoder for the block?
*/
//#[macro_use]

pub struct PersistentModifier{
    parent_id: SHA256,
}

/// PersistentModifiers have parent modifiers defined all the way upto genesis blocks.
impl PersistentModifier{
    fn new(modifier_id:  SHA256) -> Self{
        PersistentModifier {parent_id: modifier_id}
    }
}

// ToDo: Add the public ky of the miner
// ToDo: #[derive(Serialize, Deserialize, Debug)]
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
    extra_content_hash: SHA256, //for debugging purpose
    /// Mining Difficulty
    difficulty: u64,

}

impl BlockHeader{
    pub fn new(parent_id: SHA256, timestamp: u64, nonce: u32, content_root: SHA256,
           extra_content_hash: SHA256, difficulty: u64 ) -> Self{
        BlockHeader { parent: PersistentModifier::new(parent_id),
                        timestamp, nonce, content_root, extra_content_hash, difficulty }
    }

    fn difficulty(&self, difficulty_base: u32) -> bool {
        // Returns true if the hash is less than than the difficulty_base
        return true;
    }
}

impl Hashable for BlockHeader{
    fn sha256(&self) -> SHA256 {
        // ToDo: Serialize the object into a byte array
        // return the SHA256 of the byte array
        let x: [u8; 32] = [0; 32];
        return SHA256(x);
    }
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{{\n")?;
        write!(f, "  Parent Id: {}\n", self.parent.parent_id)?;
        write!(f, "  Timestamp: {}\n", self.timestamp)?;
        write!(f, "  nonce: {}\n", self.nonce)?;
        write!(f, "  content Merkle root: {}\n", self.content_root)?;
        write!(f, "  extra content hash: {}\n", self.extra_content_hash)?;
        write!(f, "}}")
    }
}