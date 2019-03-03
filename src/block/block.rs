extern crate ring;
use super::crypto::hash::{Hashable, H256};
use super::header::BlockHeader;
//use serde::{Serialize, Deserialize};
use std::fmt;


// ToDo: Encoder and decoder for the block?
// ToDo: #[derive(Serialize, Deserialize, Debug)]
#[derive(PartialEq, Clone)]
pub enum BlockType{
    Transaction,
    Proposer,
    Voter,
}

pub struct Block<T: Hashable> {
    block_header: BlockHeader,
    /// Content and its sortition proof. The content could be tx, ref or votes.
    content: T,
    sortition_proof: Vec<H256>, //Specific to Prism
    block_type: BlockType,  //Specific to Prism
}

impl<T: Hashable> Block<T>{
    pub fn get_block_type(&self) -> BlockType {return self.block_type.clone();}
}

// The generic T is used to refer all the (three) content types.
impl<T: Hashable> Block<T>{

    /// Sorititions the block into blocktype using the hash of the header
    fn sortition(hash: H256) -> BlockType {
        return BlockType::Proposer; // ToDo: Change this according to logic
    }

    pub fn new(parent_id: H256, timestamp: u64, nonce: u32, content_root: H256,
           sortition_proof: Vec<H256>, content: T, extra_content: Vec<u32>, difficulty: u64  ) -> Self {
        let block_header = BlockHeader::new(parent_id, timestamp, nonce, content_root, extra_content, difficulty);
        let block_type = Block::<T>::sortition(block_header.hash()); // Todo: should be .hash()
        Block {block_header, content, sortition_proof, block_type }
    }
}

impl<T: Hashable> fmt::Display for Block<T>  {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{{\n")?;
//        write!(f, "  Block Type: {}\n", self.block_type)?;
        // Add rest
        write!(f, "}}")
    }
}

