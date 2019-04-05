use crate::block::Block;
use crate::blockdb::BlockDatabase;
use crate::blockchain::BlockChain;
use crate::crypto::hash::Hashable;
use std::sync::Mutex;

pub fn new_block(block: Block, db: &BlockDatabase, chain: &Mutex<BlockChain>) {
    // TODO: for now, we assume that blocks appear in order
    let hash = block.hash();

    // insert the new block into the blockchain
    let mut chain = chain.lock().unwrap();
    chain.insert_node(&block);
    drop(chain);

    // insert the new block into the blockdb
    db.insert(&hash, &block).unwrap();
}

