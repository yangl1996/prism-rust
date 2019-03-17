/** Memory pool that holds unconfirmed transactions. Miner picks transactions from memory pool.
    Methods for memory pool:
    new()
    check_double_spend(transaction)
    insert_verified(transaction) //we need to check_double_spend before insert, NOTE: how to check_double_spend in concurrency?
    remove_by_hash //if a tx is confirmed, we remove it
    remove_by_prevout //for all inputs of this tx, we also need to call remove_by_prevout
    */
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Transaction, Input, IndexedTransaction};
use std::hash::{Hash, Hasher};
use bincode::serialize;
use rand::Rng;

/// Transactions memory pool
#[derive(Debug)]
pub struct MemoryPool {
    /// Transactions storage
    storage: Storage,
}

/// transactions storage
#[derive(Debug)]
struct Storage {
    /// Throughout transactions counter
    counter: u64,
    /// Total transactions size (when serialized) in bytes
    transactions_size_in_bytes: usize,
    /// By-hash storage
    by_hash: HashMap<H256, Entry>,
    /// Transactions by previous output
    by_previous_output: HashMap<HashedOutPoint, H256>,
}

/// Single entry
#[derive(Debug)]
pub struct Entry {
    /// Transaction
    pub transaction: Transaction,
    /// Transaction hash (stored for effeciency)
    pub hash: H256,
    /// Transaction size (stored for effeciency)
    pub size: usize,
    /// Throughout index of this transaction in memory pool (non persistent)
    pub storage_index: u64,
    /// Transaction fee (stored for efficiency)
    pub miner_fee: u64,
}

/// Information on current `MemoryPool` state
#[derive(Debug)]
pub struct Information {
    /// Number of transactions currently in the `MemoryPool`
    pub transactions_count: usize,
    /// Total number of bytes occupied by transactions from the `MemoryPool`
    pub transactions_size_in_bytes: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HashedOutPoint {
    /// (Previous) Transaction output point, (New) Transaction possible input
    out_point: Input,
}

impl From<Input> for HashedOutPoint {
    fn from(out_point: Input) -> Self {
        HashedOutPoint {
            out_point: out_point,
        }
    }
}

impl Hash for HashedOutPoint {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write(&serialize(&self.out_point).unwrap()[..]);
        state.finish();
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            counter: 0,
            transactions_size_in_bytes: 0,
            by_hash: HashMap::new(),
            by_previous_output: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: Entry) {
        // update pool information
        self.transactions_size_in_bytes += entry.size;

        // remember that all inputs of this transaction are spent
        for input in &entry.transaction.inputs {
            let previous_tx = self.by_previous_output.insert(input.clone().into(), entry.hash.clone());
            assert_eq!(previous_tx, None); // transaction must be verified before => no double spend TODO: Gerui: do we keep this?
        }

        // add to by_hash storage
        self.by_hash.insert(entry.hash.clone(), entry);
    }

    pub fn get_by_hash(&self, h: &H256) -> Option<&Entry> {
        self.by_hash.get(h)
    }

    pub fn contains(&self, hash: &H256) -> bool {
        self.by_hash.contains_key(hash)
    }

    pub fn is_output_spent(&self, prevout: &Input) -> bool {
        self.by_previous_output.contains_key(&prevout.clone().into())
    }

    pub fn read_by_hash(&self, h: &H256) -> Option<&Transaction> {
        self.by_hash.get(h).map(|e| &e.transaction)
    }

    pub fn remove_by_hash(&mut self, h: &H256) -> Option<Entry> {
        self.by_hash.remove(h)
            .map(|entry| {
                // update pool information
                self.transactions_size_in_bytes -= entry.size;

                // forget that all inputs of this transaction are spent
                for input in &entry.transaction.inputs {
                    let spent_in_tx = self.by_previous_output.remove(&input.clone().into())
                        .expect("by_spent_output is filled for each incoming transaction inputs; so the drained value should exist; qed");
                    assert_eq!(&spent_in_tx, h);
                }

                entry
            })
    }

    pub fn is_double_spend(&self, transaction: &Transaction) -> bool {
        for input in &transaction.inputs {
            if self.is_output_spent(input) {
                return true;
            }
        }

        false
    }

    pub fn remove_by_prevout(&mut self, prevout: &Input) -> Option<Vec<IndexedTransaction>> {
        let mut queue: VecDeque<Input> = VecDeque::new();
        let mut removed: Vec<IndexedTransaction> = Vec::new();
        queue.push_back(prevout.clone());

        while let Some(prevout) = queue.pop_front() {
            if let Some(entry_hash) = self.by_previous_output.get(&prevout.clone().into()).cloned() {
                let entry = self.remove_by_hash(&entry_hash).expect("checked that it exists line above; qed");
                queue.extend(entry.transaction.outputs.iter().enumerate().map(|(idx, _)| Input {
                    hash: entry_hash.clone(),
                    index: idx as u32,
                }));
                removed.push(IndexedTransaction::new(entry.hash, entry.transaction));
            }
        }

        Some(removed)
    }

    pub fn get_transactions_ids(&self) -> Vec<H256> {
        self.by_hash.keys().cloned().collect()
    }

    pub fn get_n_transactions_ids(&self, n: usize) -> Vec<H256> {
        let ids: Vec<H256> = self.by_hash.keys().cloned().collect();
        if ids.len() > n {
            rand::seq::sample_slice(&mut rand::thread_rng(), &ids, n)
        } else {
            ids
        }

    }

}

impl Default for MemoryPool {
    fn default() -> Self {
        MemoryPool {
            storage: Storage::new(),
        }
    }
}

impl MemoryPool {
    /// Creates new memory pool
    pub fn new() -> Self {
        MemoryPool::default()
    }

    /// Insert verified transaction to the `MemoryPool`
    pub fn insert_verified(&mut self, t: IndexedTransaction) {
        if let Some(entry) = self.make_entry(t) {
            self.storage.insert(entry);
        }
    }

    /// Removes single transaction by its hash.
    /// All descedants remain in the pool.
    pub fn remove_by_hash(&mut self, h: &H256) -> Option<Transaction> {
        self.storage.remove_by_hash(h).map(|entry| entry.transaction)
    }

    /// Checks if `transaction` spends some outputs, already spent by inpool transactions.
    pub fn is_double_spend(&self, transaction: &Transaction) -> bool {
        self.storage.is_double_spend(transaction)
    }

    /// Removes transaction (and all its descendants) which has spent given output
    pub fn remove_by_prevout(&mut self, prevout: &Input) -> Option<Vec<IndexedTransaction>> {
        self.storage.remove_by_prevout(prevout)
    }

    /// Reads single transaction by its hash.
    pub fn read_by_hash(&self, h: &H256) -> Option<&Transaction> {
        self.storage.read_by_hash(h)
    }

    /// Get transaction by hash
    pub fn get(&self, hash: &H256) -> Option<&Transaction> {
        self.storage.get_by_hash(hash).map(|entry| &entry.transaction)
    }

    /// Checks if transaction is in the mempool
    pub fn contains(&self, hash: &H256) -> bool {
        self.storage.contains(hash)
    }

    /// Returns information on `MemoryPool` (as in GetMemPoolInfo RPC)
    /// https://bitcoin.org/en/developer-reference#getmempoolinfo
    pub fn information(&self) -> Information {
        Information {
            transactions_count: self.storage.by_hash.len(),
            transactions_size_in_bytes: self.storage.transactions_size_in_bytes,
        }
    }

    pub fn get_transactions_ids(&self) -> Vec<H256> {
        self.storage.get_transactions_ids()
    }

    pub fn get_n_transactions_ids(&self, n: usize) -> Vec<H256> {
        self.storage.get_n_transactions_ids(n)
    }

    /// Returns true if output was spent
    pub fn is_spent(&self, prevout: &Input) -> bool {
        self.storage.is_output_spent(prevout)
    }

    fn make_entry(&mut self, t: IndexedTransaction) -> Option<Entry> {
        let size = self.get_transaction_size(&t.raw);
        let storage_index = self.get_storage_index();
        let miner_fee = 0;

        Some(Entry {
            transaction: t.raw,
            hash: t.hash,
            storage_index: storage_index,
            size: size,
            miner_fee: miner_fee,
        })
    }

    fn get_transaction_size(&self, t: &Transaction) -> usize {
        serialize(&t).unwrap().len()
    }

    fn get_storage_index(&mut self) -> u64 {
        self.storage.counter += 1;
        self.storage.counter
    }

}

#[cfg(test)]
pub mod tests {
    use super::MemoryPool;
    use crate::transaction::{Transaction, Input, Output};
    use crate::crypto::hash::{Hashable, H256};
    use crate::miner::fee::{FeeIsOne, FeeIsZero};
    use crate::transaction::transaction_builder::TransactionBuilder;

    #[test]
    fn test_memory_pool_insert_one_transaction() {
        let mut pool = MemoryPool::new();
        pool.insert_verified(TransactionBuilder::random_transaction_builder().into());
        assert_eq!(pool.get_transactions_ids().len(), 1);
        let id = pool.get_transactions_ids()[0];
        pool.remove_by_hash(&id);
        assert_eq!(pool.get_transactions_ids().len(), 0);
    }

    #[test]
    fn test_memory_pool_doublespend_transaction() {
        let mut pool = MemoryPool::new();
        pool.insert_verified(TransactionBuilder::random_transaction_builder().into());
        assert_eq!(pool.get_transactions_ids().len(), 1);
        let id = pool.get_transactions_ids()[0];
        pool.insert_verified(TransactionBuilder::random_transaction_builder().add_input(id, 0).into());
        assert_eq!(pool.get_transactions_ids().len(), 2);
        let tx: Transaction = TransactionBuilder::random_transaction_builder().add_input(id, 0).into();
        assert_eq!(pool.is_double_spend(&tx), true);
    }

    #[test]
    fn test_memory_pool_insert_multiple_transaction() {
        let mut pool = MemoryPool::new();
        for i in 0..20 {
            pool.insert_verified(TransactionBuilder::random_transaction_builder().into());
        }
        assert_eq!(pool.get_transactions_ids().len(), 20);
        assert_eq!(pool.get_n_transactions_ids(15).len(), 15);
        assert_eq!(pool.get_n_transactions_ids(25).len(), 20);

    }

}