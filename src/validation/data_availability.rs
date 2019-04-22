use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use std::sync::Mutex;

/// The error type for this module.
pub enum Error {
    MissingInDB,
    MissingInBlockchain,
}

/// Check whether a block is in the blockchain and the block database. Returns the block if yes,
/// and returns the cause if no.
pub fn get_block(
    block_hash: H256,
    blockchain: &Mutex<BlockChain>,
    block_db: &BlockDatabase,
) -> Result<Block, Error> {
    match block_db.get(&block_hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => {
            // check whether the block is in the database
            match b {
                None => {
                    return Err(Error::MissingInDB);
                }
                Some(block) => {
                    // check whether the block is in the blockchain
                    let blockchain = blockchain.lock().unwrap();
                    if !blockchain.check_node(block_hash) {
                        return Err(Error::MissingInBlockchain);
                    } else {
                        return Ok(block);
                    }
                }
            }
        }
    }
}

