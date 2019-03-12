use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use crate::crypto::hash::{Hashable, H256};
use super::Transaction;

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
//    /// Transactions by previous output
//    by_previous_output: HashMap<HashedOutPoint, H256>,
//    /// References storage
//    references: ReferenceStorage,
}

/// Single entry
#[derive(Debug)]
pub struct Entry {
    /// Transaction
    pub transaction: Transaction,
    /// In-pool ancestors hashes for this transaction
    pub ancestors: HashSet<H256>,
    /// Transaction hash (stored for effeciency)
    pub hash: H256,
    /// Transaction size (stored for effeciency)
    pub size: usize,
    /// Throughout index of this transaction in memory pool (non persistent)
    pub storage_index: u64,
    /// Transaction fee (stored for efficiency)
    pub miner_fee: u64,
//    /// Virtual transaction fee (a way to prioritize/penalize transaction)
//    pub miner_virtual_fee: i64,
//    /// size + Sum(size) for all in-pool descendants
//    pub package_size: usize,
//    /// miner_fee + Sum(miner_fee) for all in-pool descendants
//    pub package_miner_fee: u64,
//    /// miner_virtual_fee + Sum(miner_virtual_fee) for all in-pool descendants
//    pub package_miner_virtual_fee: i64,
}

/// Information on current `MemoryPool` state
#[derive(Debug)]
pub struct Information {
    /// Number of transactions currently in the `MemoryPool`
    pub transactions_count: usize,
    /// Total number of bytes occupied by transactions from the `MemoryPool`
    pub transactions_size_in_bytes: usize,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            counter: 0,
            transactions_size_in_bytes: 0,
            by_hash: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: Entry) {
        // update pool information
        self.transactions_size_in_bytes += entry.size;

        // add to by_hash storage
        self.by_hash.insert(entry.hash.clone(), entry);
    }

    pub fn get_by_hash(&self, h: &H256) -> Option<&Entry> {
        self.by_hash.get(h)
    }

    pub fn contains(&self, hash: &H256) -> bool {
        self.by_hash.contains_key(hash)
    }

    pub fn read_by_hash(&self, h: &H256) -> Option<&Transaction> {
        self.by_hash.get(h).map(|e| &e.transaction)
    }

    pub fn remove_by_hash(&mut self, h: &H256) -> Option<Entry> {
        self.by_hash.remove(h)
            .map(|entry| {
                // update pool information
                self.transactions_size_in_bytes -= entry.size;

                entry
            })
    }

    pub fn get_transactions_ids(&self) -> Vec<H256> {
        self.by_hash.keys().cloned().collect()
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

//    /// Insert verified transaction to the `MemoryPool`
//    pub fn insert_verified<FC: MemoryPoolFeeCalculator>(&mut self, t: IndexedTransaction, fc: &FC) {
//        if let Some(entry) = self.make_entry(t, fc) {
//            let descendants = self.storage.remove_by_parent_hash(&entry.hash);
//            self.storage.insert(entry);
//            if let Some(descendants_iter) = descendants.map(|d| d.into_iter()) {
//                for descendant in descendants_iter {
//                    if let Some(descendant_entry) = self.make_entry(descendant, fc) {
//                        self.storage.insert(descendant_entry);
//                    }
//                }
//            }
//        }
//    }

//    /// Iterator over memory pool transactions according to specified strategy
//    pub fn iter(&self, strategy: OrderingStrategy) -> MemoryPoolIterator {
//        MemoryPoolIterator::new(self, strategy)
//    }

    /// Removes single transaction by its hash.
    /// All descedants remain in the pool.
    pub fn remove_by_hash(&mut self, h: &H256) -> Option<Transaction> {
        self.storage.remove_by_hash(h).map(|entry| entry.transaction)
    }

//    /// Checks if `transaction` spends some outputs, already spent by inpool transactions.
//    pub fn check_double_spend(&self, transaction: &Transaction) -> DoubleSpendCheckResult {
//        self.storage.check_double_spend(transaction)
//    }
//
//    /// Removes transaction (and all its descendants) which has spent given output
//    pub fn remove_by_prevout(&mut self, prevout: &OutPoint) -> Option<Vec<IndexedTransaction>> {
//        self.storage.remove_by_prevout(prevout)
//    }

    /// Reads single transaction by its hash.
    pub fn read_by_hash(&self, h: &H256) -> Option<&Transaction> {
        self.storage.read_by_hash(h)
    }

//    /// Reads hash of the 'top' transaction from the `MemoryPool` using selected strategy.
//    /// Ancestors are always returned before descendant transactions.
//    pub fn read_with_strategy(&mut self, strategy: OrderingStrategy) -> Option<H256> {
//        self.storage.read_with_strategy(strategy)
//    }

//    /// Reads hashes of up to n transactions from the `MemoryPool`, using selected strategy.
//    /// Ancestors are always returned before descendant transactions.
//    /// Use this function with care, only if really needed (heavy memory usage)
//    pub fn read_n_with_strategy(&mut self, n: usize, strategy: OrderingStrategy) -> Vec<H256> {
//        self.iter(strategy).map(|entry| entry.hash.clone()).take(n).collect()
//    }

//    /// Removes the 'top' transaction from the `MemoryPool` using selected strategy.
//    /// Ancestors are always removed before descendant transactions.
//    pub fn remove_with_strategy(&mut self, strategy: OrderingStrategy) -> Option<IndexedTransaction> {
//        self.storage.remove_with_strategy(strategy)
//    }

//    /// Removes up to n transactions from the `MemoryPool`, using selected strategy.
//    /// Ancestors are always removed before descendant transactions.
//    pub fn remove_n_with_strategy(&mut self, n: usize, strategy: OrderingStrategy) -> Vec<IndexedTransaction> {
//        self.storage.remove_n_with_strategy(n, strategy)
//    }

//    /// Set miner virtual fee for transaction
//    pub fn set_virtual_fee(&mut self, h: &H256, virtual_fee: i64) {
//        self.storage.set_virtual_fee(h, virtual_fee)
//    }

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

    /// Returns TXIDs of all transactions in `MemoryPool` (as in GetRawMemPool RPC)
    /// https://bitcoin.org/en/developer-reference#getrawmempool
    pub fn get_transactions_ids(&self) -> Vec<H256> {
        self.storage.get_transactions_ids()
    }

//    /// Returns true if output was spent
//    pub fn is_spent(&self, prevout: &OutPoint) -> bool {
//        self.storage.is_output_spent(prevout)
//    }

    fn make_entry(&mut self, t: Transaction) -> Option<Entry> {
        let ancestors = self.get_ancestors(&t);
        let size = self.get_transaction_size(&t);
        let storage_index = self.get_storage_index();
        let miner_fee = 0;//we need a function calculate_fee(&t);

        // do not accept any transactions that have negative OR zero fee
        if miner_fee == 0 {
            return None;
        }
        let hash = t.hash();
        Some(Entry {
            transaction: t,
            hash: hash,
            ancestors: ancestors,
            storage_index: storage_index,
            size: size,
            miner_fee: miner_fee,
        })
    }

    fn get_ancestors(&self, t: &Transaction) -> HashSet<H256> {
        let mut ancestors: HashSet<H256> = HashSet::new();
        let ancestors_entries = t.input.iter()
            .filter_map(|input| self.storage.get_by_hash(&input.hash));
        for ancestor_entry in ancestors_entries {
            ancestors.insert(ancestor_entry.hash.clone());
            for grand_ancestor in &ancestor_entry.ancestors {
                ancestors.insert(grand_ancestor.clone());
            }
        }
        ancestors
    }

    fn get_transaction_size(&self, t: &Transaction) -> usize {
        0//TODO: t.serialized_size()
    }

    #[cfg(not(test))]
    fn get_storage_index(&mut self) -> u64 {
        self.storage.counter += 1;
        self.storage.counter
    }

    #[cfg(test)]
    fn get_storage_index(&self) -> u64 {
        (self.storage.by_hash.len() % 3usize) as u64
    }
}

