use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Input, Transaction};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;

/// transactions storage
#[derive(Debug)]
pub struct MemoryPool {
    /// Number of transactions
    counter: u64,
    /// By-hash storage
    by_hash: HashMap<H256, Entry>,
    /// Transactions by previous output
    by_input: HashMap<Input, H256>,
    /// Storage for order by storage index, it is equivalent to FIFO
    by_storage_index: BTreeMap<u64, H256>, // consider BTreeSet, why BTreeSet?
                                           // TODO: pending storage: txs whose input is in pool (or in pending?)
                                           // TODO: orphan storage: txs whose input can't be found in utxo or pool
}

#[derive(Debug, Clone)]
pub struct Entry {
    /// Transaction
    pub transaction: Transaction,
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

    pub fn insert(&mut self, tx: Transaction) {
        // assume duplicate and double spend already checked/validated
        let hash = tx.hash();
        let entry = Entry {
            transaction: tx,
            storage_index: self.counter,
        };
        self.counter += 1;

        // associate all inputs with this transaction
        for input in &entry.transaction.input {
            self.by_input.insert(input.clone(), hash);
        }

        // add to btree
        self.by_storage_index.insert(entry.storage_index, hash);

        // add to hashmap
        self.by_hash.insert(hash, entry);
    }

    fn get(&self, h: &H256) -> Option<&Entry> {
        let entry = self.by_hash.get(h)?;
        return Some(entry);
    }

    // for test
    pub fn contains(&self, h: &H256) -> bool {
        self.by_hash.contains_key(h)
    }

    // when adding tx into mempool, should check this
    pub fn is_double_spend(&self, inputs: &Vec<Input>) -> bool {
        for input in inputs {
            if self.by_input.contains_key(input) {
                return true;
            }
        }
        return false;
    }

    fn remove_and_get(&mut self, h: &H256) -> Option<Entry> {
        let entry = self.by_hash.remove(h)?;
        for input in &entry.transaction.input {
            self.by_input.remove(&input.clone());
        }
        self.by_storage_index.remove(&entry.storage_index);
        return Some(entry);
    }

    // only need this remove function, don't need remove_by_hash
    pub fn remove_by_input(&mut self, prevout: &Input) {
        //use a deque to recursively remove, in case there are multi level dependency between txs.
        let mut queue: VecDeque<Input> = VecDeque::new();
        queue.push_back(prevout.clone());

        while let Some(prevout) = queue.pop_front() {
            if let Some(entry_hash) = self.by_input.get(&prevout) {
                let entry_hash = *entry_hash;
                let entry = self.remove_and_get(&entry_hash).unwrap();
                for (index, output) in entry.transaction.output.iter().enumerate() {
                    queue.push_back(Input {
                        hash: entry_hash,
                        index: index as u32,
                        value: output.value,
                        recipient: output.recipient,
                    });
                }
            }
        }
    }

    /// get n transaction by fifo
    pub fn get_transactions(&self, n: usize) -> Vec<Transaction> {
        self.by_storage_index
            .values()
            .take(n)
            .map(|hash| self.get(hash).unwrap().transaction.clone())
            .collect()
    }

    /// get size/length
    pub fn len(&self) -> usize {
        self.by_hash.len()
    }
}

#[cfg(test)]
pub mod tests {
    // TODO: add more tests.

    use super::MemoryPool;
    use crate::crypto::generator as crypto_generator;
    use crate::crypto::hash::{Hashable, H256};
    use crate::transaction::Transaction;
    use crate::transaction::{generator as tx_generator, Input};

    #[test]
    fn insert_remove_transactions() {
        let mut pool = MemoryPool::new();
        let tx = tx_generator::random();
        let h = tx.hash();
        pool.insert(tx.clone());
        assert_eq!(pool.by_hash.len(), 1);
        assert_eq!(pool.by_input.len(), tx.input.len());
        assert_eq!(pool.by_storage_index.len(), 1);
        pool.remove_by_input(&tx.input[0]);
        assert_eq!(pool.by_hash.len(), 0);
        assert_eq!(pool.by_input.len(), 0);
        assert_eq!(pool.by_storage_index.len(), 0);
        for i in 1..=20 {
            let tx = tx_generator::random();
            pool.insert(tx.clone());
            assert_eq!(pool.by_hash.len(), i);
            assert_eq!(pool.by_storage_index.len(), i);
        }
    }

    #[test]
    fn check_duplicate_doublespend() {
        let mut pool = MemoryPool::new();
        let tx = tx_generator::random();
        let h = tx.hash();
        pool.insert(tx.clone());
        assert!(pool.contains(&h));
        assert!(pool.is_double_spend(&tx.input));
    }

    #[test]
    fn remove_by_input() {
        let mut pool = MemoryPool::new();
        let first_tx = tx_generator::random();
        let mut input = vec![Input {
            hash: first_tx.hash(),
            index: 0,
            value: first_tx.output[0].value,
            recipient: first_tx.output[0].recipient,
        }];
        //although for now we don't want to add tx into mempool if it's inputs are also in mempool
        //let's add them and test remove
        for i in 1..=20 {
            let tx = Transaction {
                input: input.clone(),
                ..tx_generator::random()
            };
            pool.insert(tx.clone());
            assert_eq!(pool.by_hash.len(), i);
            assert_eq!(pool.by_storage_index.len(), i);
            input = vec![Input {
                hash: tx.hash(),
                index: 0,
                value: tx.output[0].value,
                recipient: tx.output[0].recipient,
            }];
        }
        //one remove should remove all correlated transactions
        pool.remove_by_input(&Input {
            hash: first_tx.hash(),
            index: 0,
            value: first_tx.output[0].value,
            recipient: first_tx.output[0].recipient,
        });
        assert_eq!(pool.by_hash.len(), 0);
        assert_eq!(pool.by_input.len(), 0);
    }

    #[test]
    fn fifo() {
        let mut pool = MemoryPool::new();
        let mut v = vec![];
        for _i in 0..20 {
            let tx: Transaction = tx_generator::random();
            v.push(tx.hash());
            pool.insert(tx);
        }
        //test the fifo property: we get the first i txs.
        for i in 0..20 {
            assert_eq!(
                pool.get_transactions(i)
                    .iter()
                    .map(|tx| tx.hash())
                    .collect::<Vec<H256>>()[..],
                v[..i]
            )
        }
        //if we pass in a larger integer, we get all transactions in mempool.
        assert_eq!(pool.get_transactions(25).len(), 20);
    }

}
