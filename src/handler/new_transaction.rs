use crate::miner::memory_pool::MemoryPool;
use crate::transaction::Transaction;
use std::sync::Mutex;
use crate::crypto::hash::Hashable;

/// Handler for new transaction
// We may want to add the result of memory pool check
pub fn new_transaction(transaction: Transaction, mempool: &Mutex<MemoryPool>) {
    let mut mempool = mempool.lock().unwrap();
    // memory pool check
    if !mempool.contains(&transaction.hash()) && !mempool.is_double_spend(&transaction.input) {
        // if check passes, insert the new transaction into the mempool
        mempool.insert(transaction);
    }
}
