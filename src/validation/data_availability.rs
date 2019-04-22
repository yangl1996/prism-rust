use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};

use super::BlockDataAvailability;
use std::sync::Mutex;

/// Data availability: Checks if the block_hash is present in database and blockchain.
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
