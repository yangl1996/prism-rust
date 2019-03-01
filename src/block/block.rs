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
    parent: PersistentModifier,
    timestamp: u8,
    nonce: u8,
    content_merkle_root: SHA256,
    extra_content_hash: SHA256, //for debugging purpose
}

impl BlockHeader{
    fn new(parent_id: SHA256, timestamp: u8, nonce: u8, content_merkle_root: SHA256, extra_content_hash: SHA256 ) -> Self{
        BlockHeader { parent: PersistentModifier::new(parent_id),
                        timestamp,nonce, content_merkle_root, extra_content_hash }
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
        write!(f, "  content Merkle root: {}\n", self.content_merkle_root)?;
        write!(f, "  extra content hash: {}\n", self.extra_content_hash)?;
        write!(f, "}}")
    }
}


pub enum BlockType{
    Transaction,
    Proposer,
    Voter,
}

//ToDo: #[derive(Serialize, Deserialize, Debug)]
pub struct Block<T: Hashable> {
    modifier_id: SHA256,
    block_type: BlockType,
    block_header: BlockHeader,
    content_merkle_proof: Vec<hash::SHA256>, //missing lifetime specifier?
    content: T,
    extra_content: [u8; 50] // 50 is a random number
}


// The hashable generic T is used to define all the (three) types of blocks.
impl<T: Hashable> Block<T>{
    fn id_to_block_type(modifier_id: SHA256) -> BlockType {
        return BlockType::Proposer;
    }

    fn new(parent_id: SHA256, timestamp: u8, nonce: u8, content_merkle_root: SHA256,
           content_merkle_proof: Vec<hash::SHA256>, content: T, extra_content: [u8; 50] ) -> Self {

        let extra_content_hash = ring::digest::digest(&ring::digest::SHA256, &extra_content).into();
        let block_header = BlockHeader::new(parent_id, timestamp, nonce, content_merkle_root, extra_content_hash);

        let modifier_id = block_header.sha256();
        let block_type = Block::<T>::id_to_block_type(modifier_id);
        Block { modifier_id, block_type, block_header, content_merkle_proof, content, extra_content }
    }

}


impl<T: Hashable> fmt::Display for Block<T>  {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{{\n")?;
        write!(f, "  Modifier Id: {}\n", self.modifier_id)?;
//        write!(f, "  Block Type: {}\n", self.block_type)?;
        // Add rest
        write!(f, "}}")
    }
}