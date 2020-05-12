use crate::block::{Block, Content};
use crate::crypto::hash::H256;
use std::collections::HashSet;
use crate::blockchain::{BlockChain, NewBlock};
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::miner::memory_pool::MemoryPool;
use crate::ledger_manager::index::LedgerIndex;
use crate::network::server::Handle as ServerHandle;

use std::sync::Mutex;

pub fn new_validated_block(
    block: &Block,
    hash: H256,
    mempool: &Mutex<MemoryPool>,
    _blockdb: &BlockDatabase,
    chain: &BlockChain,
    _server: &ServerHandle,
    unconfirmed_set: &Mutex<HashSet<H256>>,
) -> NewBlock {
    PERFORMANCE_COUNTER.record_process_block(&block);
    if let Content::Proposer(_) = &block.content {
        let mut ledger_ptr = unconfirmed_set.lock().unwrap();
        ledger_ptr.insert(hash);
        drop(ledger_ptr);
    }

    // if this block is a transaction, remove transactions from mempool
    if let Content::Transaction(content) = &block.content {
        let mut mempool = mempool.lock().unwrap();
        for tx in &content.transactions {
            mempool.remove_by_hash(&tx.hash());
            // the inputs have been used here, so remove all transactions in the mempool that
            // tries to use the input again.
            for input in tx.input.iter() {
                mempool.remove_by_input(input);
            }
        }
        drop(mempool);
    }

    // insert the new block into the blockchain
    return chain.insert_block(&block).unwrap();
}
