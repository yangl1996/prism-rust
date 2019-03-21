/** Memory pool that holds unconfirmed transactions. Miner picks transactions from memory pool.
    Methods for memory pool:
    new()
    is_double_spend(transaction)
    insert_verified(transaction) //we need to check_double_spend before insert, NOTE: how to check_double_spend in concurrency?
    remove_by_hash //if a tx is confirmed, we remove it
    remove_by_prevout //for all inputs of this tx, we also need to call remove_by_prevout. to discuss: remove_by_prevout can replace remove_by_hash?
    get_n_transactions(n) //get n transactions
    get_n_transactions_hash(n) //just hash of transactions
    */
use std::collections::HashMap;
use std::collections::BTreeMap;
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
    //future: pending storage: txs whose input is in pool (or in pending?)
    //future: orphan storage: txs whose input can't be found in utxo or pool
}

/// transactions storage
#[derive(Debug)]
struct Storage {
    /// Throughout transactions counter
    counter: u64,
    /// By-hash storage
    by_hash: HashMap<H256, Entry>,
    /// Transactions by previous output
    by_previous_output: HashMap<HashedOutPoint, H256>,
    /// Storage for order by storage index, it is equivalent to FIFO
    by_storage_index: BTreeMap<u64, H256>,
}

/// Single entry
#[derive(Debug)]
pub struct Entry {
    /// Transaction
    pub transaction: Transaction,
    /// Transaction hash (stored for effeciency)
    pub hash: H256,
    /// Throughout index of this transaction in memory pool (non persistent)
    pub storage_index: u64,
    /// Transaction fee (stored for efficiency)
    pub miner_fee: u64,
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
        state.write(&serialize(&self.out_point).unwrap());
        state.finish();
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            counter: 0,
            by_hash: HashMap::new(),
            by_previous_output: HashMap::new(),
            by_storage_index: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, entry: Entry) {

        // remember that all inputs of this transaction are spent
        for input in &entry.transaction.input {
            let previous_tx = self.by_previous_output.insert(input.clone().into(), entry.hash.clone());
            assert_eq!(previous_tx, None); // transaction must be verified before => no double spend TODO: Gerui: do we keep this?
        }

        // add to by_storage_index
        self.by_storage_index.insert(entry.storage_index, entry.hash.clone());

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
        let entry = self.by_hash.remove(h)
            .map(|entry| {

                // forget that all inputs of this transaction are spent
                for input in &entry.transaction.input {
                    let spent_in_tx = self.by_previous_output.remove(&input.clone().into())
                        .expect("by_spent_output is filled for each incoming transaction inputs; so the drained value should exist; qed");
                    assert_eq!(&spent_in_tx, h);
                }

                self.by_storage_index.remove(&entry.storage_index);

                entry
            });
        entry
    }

    pub fn is_double_spend(&self, transaction: &Transaction) -> bool {
        for input in &transaction.input {
            if self.is_output_spent(input) {
                return true;
            }
        }

        false
    }

    pub fn remove_by_prevout(&mut self, prevout: &Input) -> Option<Vec<IndexedTransaction>> {
        //use a deque to recursively remove, in case there are multi level dependency between txs.
        let mut queue: VecDeque<Input> = VecDeque::new();
        let mut removed: Vec<IndexedTransaction> = Vec::new();
        queue.push_back(prevout.clone());

        while let Some(prevout) = queue.pop_front() {
            if let Some(entry_hash) = self.by_previous_output.get(&prevout.clone().into()).cloned() {
                let entry = self.remove_by_hash(&entry_hash).expect("checked that it exists line above; qed");
                queue.extend(entry.transaction.output.iter().enumerate().map(|(hx, _)| Input {
                    hash: entry_hash.clone(),
                    index: hx as u32,
                }));
                removed.push(IndexedTransaction::new(entry.hash, entry.transaction));
            }
        }

        Some(removed)
    }

    pub fn get_transactions_hash(&self) -> Vec<H256> {
        self.by_storage_index.values().cloned().collect()
    }

    pub fn get_n_random_transactions_hash(&self, n: usize) -> Vec<H256> {
        let hashes: Vec<H256> = self.by_hash.keys().cloned().collect();
        if hashes.len() > n {
            rand::seq::sample_slice(&mut rand::thread_rng(), &hashes, n)
        } else {
            hashes
        }

    }

    /// get n transaction hashes by fifo, similar to below
    pub fn get_n_transactions_hash(&self, n: usize) -> Vec<H256> {
        self.by_storage_index.values().take(n).cloned().collect()

    }

    /// get n transaction by fifo
    pub fn get_n_transactions(&self, n: usize) -> Vec<Transaction> {
        self.by_storage_index.values().take(n).map(|h|self.read_by_hash(h).unwrap().clone()).collect()
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
        Self::default()
    }

    /// Insert verified transaction to the `MemoryPool`
    /// Important: must check is_double_spend before insert a transaction.
    pub fn insert_verified(&mut self, t: IndexedTransaction) {
        if let Some(entry) = self.make_entry(t) {
            self.storage.insert(entry);
        }
    }

    /// Removes single transaction by its hash.
    /// when a transaction is confirmed, call this and also call remove_by_prevout(input) for all its inputs
    pub fn remove_by_hash(&mut self, h: &H256) -> Option<Transaction> {
        self.storage.remove_by_hash(h).map(|entry| entry.transaction)
    }

    /// Checks if `transaction` double spends some outputs, already spent by inpool transactions.
    /// Important: must check this before insert a transaction.
    pub fn is_double_spend(&self, transaction: &Transaction) -> bool {
        self.storage.is_double_spend(transaction)
    }

    /// Removes transaction (and all its descendants) which has spent given output
    /// Important: when a transaction is confirmed, call remove_by_prevout(input) for all input in
    /// the confirmed transaction. This eliminates unwanted conflicting (double-spending)
    /// transactions.
    pub fn remove_by_prevout(&mut self, prevout: &Input) -> Option<Vec<IndexedTransaction>> {
        self.storage.remove_by_prevout(prevout)
    }

//We don't use it for now
//    /// Get transaction by hash
//    pub fn get(&self, hash: &H256) -> Option<&Transaction> {
//        self.storage.get_by_hash(hash).map(|entry| &entry.transaction)
//    }

    /// Checks if transaction is in the mempool
    pub fn contains(&self, hash: &H256) -> bool {
        self.storage.contains(hash)
    }

    pub fn get_transactions_hash(&self) -> Vec<H256> {
        self.storage.get_transactions_hash()
    }

    pub fn get_n_transactions_hash(&self, n: usize) -> Vec<H256> {
        self.storage.get_n_transactions_hash(n)
    }

    pub fn get_n_transactions(&self, n: usize) -> Vec<Transaction> {
        self.storage.get_n_transactions(n)
    }

    /// Returns true if output was spent by some inpool transaction
    pub fn is_spent(&self, prevout: &Input) -> bool {
        self.storage.is_output_spent(prevout)
    }

    fn make_entry(&mut self, t: IndexedTransaction) -> Option<Entry> {
        let storage_index = self.get_storage_index();
        let miner_fee = 0;

        Some(Entry {
            transaction: t.raw,
            hash: t.hash,
            storage_index: storage_index,
            miner_fee: miner_fee,
        })
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
    use crate::transaction::generator;

    #[test]
    fn test_memory_pool_insert_one_transaction() {
        let mut pool = MemoryPool::new();
        pool.insert_verified(generator::random_transaction_builder().into());
        assert_eq!(pool.get_transactions_hash().len(), 1);
        let h = pool.get_transactions_hash()[0];
        pool.remove_by_hash(&h);
        assert_eq!(pool.get_transactions_hash().len(), 0);
    }

    #[test]
    fn test_memory_pool_doublespend_transaction() {
        let mut pool = MemoryPool::new();
        pool.insert_verified(generator::random_transaction_builder().into());
        assert_eq!(pool.get_transactions_hash().len(), 1);
        let h = pool.get_transactions_hash()[0];
        let tx: Transaction = generator::random_transaction_builder().add_input(h, 0).into();
        pool.insert_verified(tx.clone().into());
        assert_eq!(pool.get_transactions_hash().len(), 2);
        let tx_2: Transaction = generator::random_transaction_builder().add_input(h, 0).into();
        assert_eq!(pool.is_double_spend(&tx), true);
        assert_eq!(pool.is_double_spend(&tx_2), true);
    }

    #[test]
    fn test_memory_pool_multiple_transaction_and_fifo() {
        let mut pool = MemoryPool::new();
        let mut v = vec![];
        for i in 0..20 {
            let tx: Transaction = generator::random_transaction_builder().into();
            v.push(tx.hash());
            pool.insert_verified(tx.into());
        }
        assert_eq!(pool.get_transactions_hash().len(), 20);
        assert_eq!(pool.get_n_transactions_hash(15).len(), 15);
        assert_eq!(pool.get_n_transactions_hash(15)[..], v[..15]);//test the fifoproperty: we get the first 15 txs.
        assert_eq!(pool.get_n_transactions_hash(25).len(), 20);
        assert_eq!(pool.get_n_transactions(15).len(), 15);
        assert_eq!(pool.get_n_transactions(25).len(), 20);
    }

}
