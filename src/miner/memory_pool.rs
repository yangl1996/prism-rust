use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Transaction, Input, IndexedTransaction};
use std::hash::{Hash, Hasher};
use bincode::serialize;
use crate::miner::fee::MemoryPoolFeeCalculator;

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


/// Result of checking double spend with
#[derive(Debug, PartialEq)]
pub enum DoubleSpendCheckResult {
    /// No double spend
    NoDoubleSpend,
    /// Input {self.1, self.2} of new transaction is already spent in previous final memory-pool transaction {self.0}
    DoubleSpend(H256, H256, u32),
    /// Some inputs of new transaction are already spent by non-final memory-pool transactions
    NonFinalDoubleSpend(NonFinalDoubleSpendSet),
}

/// Set of transaction outputs, which can be replaced if newer transaction
/// replaces non-final transaction in memory pool
#[derive(Debug, PartialEq)]
pub struct NonFinalDoubleSpendSet {
    /// Double-spend outputs (outputs of newer transaction, which are also spent by nonfinal transactions of mempool)
    pub double_spends: HashSet<HashedOutPoint>,
    /// Outputs which also will be removed from memory pool in case of newer transaction insertion
    /// (i.e. outputs of nonfinal transactions && their descendants)
    pub dependent_spends: HashSet<HashedOutPoint>,
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
            assert_eq!(previous_tx, None); // transaction must be verified before => no double spend
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

    pub fn check_double_spend(&self, transaction: &Transaction) -> DoubleSpendCheckResult {
        let mut double_spends: HashSet<HashedOutPoint> = HashSet::new();
        let mut dependent_spends: HashSet<HashedOutPoint> = HashSet::new();

        for input in &transaction.inputs {
            // find transaction that spends the same output
            let prevout: HashedOutPoint = input.clone().into();
            if let Some(entry_hash) = self.by_previous_output.get(&prevout).cloned() {
                // check if this is final transaction. If so, that's a potential double-spend error
                let entry = self.by_hash.get(&entry_hash).expect("checked that it exists line above; qed");
                if false {//entry.transaction.is_final() { we don't have is_final function
                    return DoubleSpendCheckResult::DoubleSpend(entry_hash,	 prevout.out_point.hash, prevout.out_point.index);
                }
                // else remember this double spend
                double_spends.insert(prevout.clone());
                // and 'virtually' remove entry && all descendants from mempool
                let mut queue: VecDeque<HashedOutPoint> = VecDeque::new();
                queue.push_back(prevout);
                while let Some(dependent_prevout) = queue.pop_front() {
                    // if the same output is already spent with another in-pool transaction
                    if let Some(dependent_entry_hash) = self.by_previous_output.get(&dependent_prevout).cloned() {
                        let dependent_entry = self.by_hash.get(&dependent_entry_hash).expect("checked that it exists line above; qed");
                        let dependent_outputs: Vec<_> = dependent_entry.transaction.outputs.iter().enumerate().map(|(idx, _)| Input {
                            hash: dependent_entry_hash.clone(),
                            index: idx as u32,
                        }.into()).collect();
                        dependent_spends.extend(dependent_outputs.clone());
                        queue.extend(dependent_outputs);
                    }
                }
            }
        }

        if double_spends.is_empty() {
            DoubleSpendCheckResult::NoDoubleSpend
        } else {
            DoubleSpendCheckResult::NonFinalDoubleSpend(NonFinalDoubleSpendSet {
                double_spends: double_spends,
                dependent_spends: dependent_spends,
            })
        }
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
    pub fn insert_verified<FC: MemoryPoolFeeCalculator>(&mut self, t: IndexedTransaction, fc: &FC) {
        if let Some(entry) = self.make_entry(t, fc) {
            self.storage.insert(entry);
        }
    }

    /// Removes single transaction by its hash.
    /// All descedants remain in the pool.
    pub fn remove_by_hash(&mut self, h: &H256) -> Option<Transaction> {
        self.storage.remove_by_hash(h).map(|entry| entry.transaction)
    }

    /// Checks if `transaction` spends some outputs, already spent by inpool transactions.
    pub fn check_double_spend(&self, transaction: &Transaction) -> DoubleSpendCheckResult {
        self.storage.check_double_spend(transaction)
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

    /// Returns TXIDs of all transactions in `MemoryPool` (as in GetRawMemPool RPC)
    /// https://bitcoin.org/en/developer-reference#getrawmempool
    pub fn get_transactions_ids(&self) -> Vec<H256> {
        self.storage.get_transactions_ids()
    }

    /// Returns true if output was spent
    pub fn is_spent(&self, prevout: &Input) -> bool {
        self.storage.is_output_spent(prevout)
    }

    fn make_entry<FC: MemoryPoolFeeCalculator>(&mut self, t: IndexedTransaction, fc: &FC) -> Option<Entry> {
        let size = self.get_transaction_size(&t.raw);
        let storage_index = self.get_storage_index();
        let miner_fee = fc.calculate(self, &t.raw);

        Some(Entry {
            transaction: t.raw,
            hash: t.hash,
            storage_index: storage_index,
            size: size,
            miner_fee: miner_fee,
        })
    }

//    fn get_ancestors(&self, t: &Transaction) -> HashSet<H256> {
//        let mut ancestors: HashSet<H256> = HashSet::new();
//        let ancestors_entries = t.inputs.iter()
//            .filter_map(|input| self.storage.get_by_hash(&input.hash));
//        for ancestor_entry in ancestors_entries {
//            ancestors.insert(ancestor_entry.hash.clone());
//            for grand_ancestor in &ancestor_entry.ancestors {
//                ancestors.insert(grand_ancestor.clone());
//            }
//        }
//        ancestors
//    }

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

    #[test]
    fn test_memory_pool_insert_one_transaction() {
        let mut pool = MemoryPool::new();
        pool.insert_verified(default_tx().into(), &FeeIsZero);
        assert_eq!(pool.get_transactions_ids().len(), 1);
        let id = pool.get_transactions_ids()[0];
        println!("{:?}", pool.get(&id));
        println!("{:?}", (&id));
        pool.remove_by_hash(&id);
        assert_eq!(pool.get_transactions_ids().len(), 0);

    }

    #[test]
    fn test_memory_pool_insert_two_transaction() {
        let mut pool = MemoryPool::new();
        pool.insert_verified(default_tx().into(), &FeeIsZero);
        assert_eq!(pool.get_transactions_ids().len(), 1);
        let id = pool.get_transactions_ids()[0];
        pool.insert_verified(fake_tx(id.clone()).into(), &FeeIsZero);
        println!("{:?}",pool.information());
        assert_eq!(pool.get_transactions_ids().len(), 2);

    }

    #[test]
    fn test_memory_pool_insert_doublespend_transaction() {//should fail
        let mut pool = MemoryPool::new();
        pool.insert_verified(default_tx().into(), &FeeIsZero);
        assert_eq!(pool.get_transactions_ids().len(), 1);
        let id = pool.get_transactions_ids()[0];
        pool.insert_verified(fake_tx(id.clone()).into(), &FeeIsZero);
        assert_eq!(pool.get_transactions_ids().len(), 2);
        pool.insert_verified(fake2_tx(id.clone()).into(), &FeeIsZero);

    }

    fn empty_tx() -> Transaction {
        Transaction { inputs: vec![], outputs: vec![], signatures: vec![]}
    }

    fn default_tx() -> Transaction {
        Transaction {
            inputs: vec![Input{ hash: H256([0u128;2]), index: 0}],
            outputs: vec![Output{ value: 2, recipient: H256([1u128;2])}],
            signatures: vec![]}
    }

    fn fake_tx(input: H256) -> Transaction {
        Transaction {
            inputs: vec![Input{ hash: input, index: 0}],
            outputs: vec![Output{ value: 2, recipient: H256([3u128;2])}],
            signatures: vec![]}
    }

    fn fake2_tx(input: H256) -> Transaction {
        Transaction {
            inputs: vec![Input{ hash: input, index: 0}],
            outputs: vec![Output{ value: 2, recipient: H256([4u128;2])}],
            signatures: vec![]}
    }
}