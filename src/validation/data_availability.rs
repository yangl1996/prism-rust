use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};

use std::sync::Mutex;

/// The return type for the func below
pub enum BlockDataAvailability {
    NotInDB,         // If: Block is not present in DB
    NotInBlockchain, // Else If: Block is not present in blockchain
    Block(Block),    // If the block passes all the data availability checks
}

/// Data availability: Checks if the block_hash is present in database and blockchain.
/// This function is paramount in defending against data availability attacks
pub fn get_available_block(
    block_hash: H256,
    blockchain: &Mutex<BlockChain>,
    block_db: &BlockDatabase,
) -> BlockDataAvailability {
    match block_db.get(&block_hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => {
            match b {
                // 1. Data availability: Check if the block is in database
                None => {
                    unimplemented!("The parent block doesnt exist in db.");
                    return BlockDataAvailability::NotInDB;
                }
                Some(block) => {
                    let blockchain = blockchain.lock().unwrap();
                    // 2. Data availability: Check if the block is in the blockchain
                    if !blockchain.check_node(block_hash) {
                        return BlockDataAvailability::NotInBlockchain;
                    } else {
                        return BlockDataAvailability::Block(block);
                    }
                }
            }
        }
    }
}
