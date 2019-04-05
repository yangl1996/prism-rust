use crate::transaction::Transaction;
use crate::miner::memory_pool::MemoryPool;
use crate::crypto::hash::Hashable;
use std::sync::Mutex;

pub fn new_transaction(transaction: Transaction, mempool: &Mutex<MemoryPool>) {
    // insert the new transaction into the mempool
    let mut mempool = mempool.lock().unwrap();
    mempool.insert(transaction);
    drop(mempool);
}

