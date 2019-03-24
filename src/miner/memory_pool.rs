use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Transaction, Input};
use bincode::serialize;
use rand::Rng;


/// transactions storage
#[derive(Debug)]
struct MemoryPool {
    /// Number of transactions
    counter: u64,
    /// By-hash storage
    by_hash: HashMap<H256, Entry>,
    /// Transactions by previous output
    by_input: HashMap<Input, H256>,
    /// Storage for order by storage index, it is equivalent to FIFO
    by_storage_index: BTreeMap<u64, H256>,  // TODO: consider BTreeSet
    // TODO: pending storage: txs whose input is in pool (or in pending?)
    // TODO: orphan storage: txs whose input can't be found in utxo or pool
}

#[derive(Debug, Clone)]
pub struct Entry {
    /// Transaction
    pub transaction: Transaction,
    /// Transaction hash
    pub hash: H256,
    /// counter of the block
    storage_index: u64,
}

impl MemoryPool {
    pub fn new() -> Self {
        Self {
            counter: 0,
            by_hash: HashMap::new(),
            by_input: HashMap::new(),
            by_storage_index: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, txn: Transaction) {
        // TODO: handle duplicate and double spend
        let hash = txn.hash();
        let entry = Entry {
            transaction: txn,
            hash: hash,
            storage_index: self.counter,
        };
        self.counter += 1;

        // associate all inputs with this transaction
        for input in &entry.transaction.input {
            self.by_input.insert(input.clone(), entry.hash);
        }

        // add to btree
        self.by_storage_index.insert(entry.storage_index, entry.hash);

        // add to hashmap
        self.by_hash.insert(entry.hash, entry);
    }

    pub fn get(&self, h: &H256) -> Option<&Entry> {
        let entry = self.by_hash.get(h)?;
        return Some(entry);
    }

    pub fn remove(&mut self, h: &H256) -> Option<Entry> {
        let entry = self.by_hash.remove(h)?;
        for input in &entry.transaction.input {
            self.by_input.remove(&input.clone());
        }
        self.by_storage_index.remove(&entry.storage_index);
        return Some(entry);
    }

    pub fn contains(&self, hash: &H256) -> bool {
        self.by_hash.contains_key(hash)
    }

    pub fn remove_by_input(&mut self, prevout: &Input) {
        //use a deque to recursively remove, in case there are multi level dependency between txs.
        let mut queue: VecDeque<Input> = VecDeque::new();
        queue.push_back(prevout.clone());

        while let Some(prevout) = queue.pop_front() {
            if let Some(entry_hash) = self.by_input.get(&prevout) {
                let entry_hash = *entry_hash;
                let entry = self.remove(&entry_hash).unwrap();
                let num_out = entry.transaction.output.len();
                for out_idx in 0..num_out {
                    queue.push_back(Input {
                        hash: entry_hash,
                        index: out_idx as u32,
                    });
                }
            }
        }
        return;
    }

    // TODO: get random_n removed for now

    /// get n transaction by fifo
    pub fn get_transactions(&self, n: usize) -> Vec<Entry> {
        self.by_storage_index.values().take(n).map(|hash|self.get(hash).unwrap().clone()).collect()
    }
}


#[cfg(test)]
pub mod tests {
    // TODO: add more tests.

    use super::MemoryPool;
    use crate::transaction::{Transaction, Input, Output};
    use crate::crypto::hash::{Hashable, H256};
    use crate::transaction::generator;

    #[test]
    fn insert_one_transaction() {
        let mut pool = MemoryPool::new();
        pool.insert(generator::random());
        assert_eq!(pool.by_hash.len(), 1);
    }

    // TODO: fix this
    /*
    #[test]
    fn fifo() {
        let mut pool = MemoryPool::new();
        let mut v = vec![];
        for i in 0..20 {
            let tx: Transaction = generator::random();
            v.push(tx.hash());
            pool.insert(tx);
        }
        assert_eq!(pool.by_hash.len(), 20);
        assert_eq!(pool.by_input.len(), 20);
        assert_eq!(pool.by_storage_index.len(), 20);

        assert_eq!(pool.get_n_transactions_hash(15).len(), 15);
        assert_eq!(pool.get_n_transactions_hash(15)[..], v[..15]);//test the fifoproperty: we get the first 15 txs.
        assert_eq!(pool.get_n_transactions_hash(25).len(), 20);
        assert_eq!(pool.get_n_transactions(15).len(), 15);
        assert_eq!(pool.get_n_transactions(25).len(), 20);
    }
    */

}
