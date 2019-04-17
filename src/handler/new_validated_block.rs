use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use std::sync::Mutex;
use crate::miner::memory_pool::MemoryPool;

pub fn new_validated_block(block: Block, mempool: &Mutex<MemoryPool>, db: &BlockDatabase, chain: &Mutex<BlockChain>) {
    // TODO: for now, we assume that blocks appear in order

    // if this block is a tx_block, remove transactions from mempool
    match &block.content {
        Content::Transaction(content) => {
            let mut mempool = mempool.lock().unwrap();
            for tx in content.transactions.iter() {
                for input in tx.input.iter() {
                    mempool.remove_by_input(input);
                }
            }
            drop(mempool);
        },
        _ => ()
    };

    // insert the new block into the blockchain
    let mut chain = chain.lock().unwrap();
    chain.insert_node(&block);
    drop(chain);

    // insert the new block into the blockdb
    db.insert(&block).unwrap();
}
