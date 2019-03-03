/*
The blockheader and block struct is defined along with their initialization and other functions
*/


extern crate ring;
use crate::hash::{self, Hashable, SHA256};
use serde::{Serialize, Deserialize};
use std::fmt;
use super::header::BlockHeader;

/*
ToDo: Encoder and decoder for the block?
*/
//#[macro_use]

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

    pub fn new(parent_id: SHA256, timestamp: u64, nonce: u32, content_root: SHA256,
           content_merkle_proof: Vec<hash::SHA256>, content: T, extra_content: [u8; 50], difficulty: u64  ) -> Self {

        let extra_content_hash = ring::digest::digest(&ring::digest::SHA256, &extra_content).into();
        let block_header = BlockHeader::new(parent_id, timestamp, nonce,
                                            content_root, extra_content_hash, difficulty);

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