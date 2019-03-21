use crate::crypto::hash::{H256, Hashable};
use std::collections::{HashSet, HashMap};
use crate::transaction::Transaction;

#[derive(Default, Debug)]
pub struct StateStorage {
    /// Stores state/utxo ( transaction_hash, indices:=set(index) ) by a hashmap
    by_transaction_hash: HashMap<H256, HashSet<u32>>,
    // for now it does not store the value of the coin
}

impl StateStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// insert only one index of a transaction
    pub fn insert(&mut self, hash: &H256, index: &u32) {
        self.by_transaction_hash.entry(hash.clone()).or_insert_with(HashSet::new).insert(index.clone());
    }

    /// insert every output of a transaction, this transaction should not previously be in state
    pub fn add(&mut self, tx: &Transaction) {
        let hash = tx.hash();
        if self.by_transaction_hash.contains_key(&hash) {
            panic!("adding a transaction already in state!");
        }
        self.by_transaction_hash.insert(hash, (0..tx.output.len() as u32).collect());
    }

    pub fn contains(&self, hash: &H256, index: &u32) -> bool {
        self.by_transaction_hash.get(hash).map_or(false, |indices|indices.contains(index))
    }

    pub fn remove(&mut self, hash: &H256, index: &u32) {
        self.by_transaction_hash.get_mut(hash).map(|indices|indices.remove(index));
        if let Some(indices) = self.by_transaction_hash.get(hash) {
            if indices.is_empty() {
                self.by_transaction_hash.remove(hash);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.by_transaction_hash.is_empty()
    }
}

#[cfg(test)]
pub mod tests {
    use super::StateStorage;
    use crate::crypto::generator as crypto_generator;
    use crate::transaction::{Transaction, generator};
    use crate::crypto::hash::Hashable;

    #[test]
    fn test_state_basic() {
        let mut state = StateStorage::new();
        assert_eq!(state.by_transaction_hash.len(), 0);
        let h = crypto_generator::h256();
        state.insert(&h, &0);
        assert_eq!(state.by_transaction_hash.len(), 1);
        state.insert(&h, &1);
        assert_eq!(state.by_transaction_hash.len(), 1);
        assert_eq!(state.contains(&h,&0), true);
        assert_eq!(state.contains(&h,&1), true);
        assert_eq!(state.contains(&h,&2), false);
        assert_eq!(state.contains(&crypto_generator::h256(),&0), false);
        state.remove(&h, &1);
        assert_eq!(state.contains(&h,&0), true);
        assert_eq!(state.contains(&h,&1), false);
        assert_eq!(state.contains(&h,&2), false);
        state.remove(&h, &0);
        assert_eq!(state.contains(&h,&0), false);
        assert_eq!(state.contains(&h,&1), false);
        assert_eq!(state.contains(&h,&2), false);
        assert_eq!(state.by_transaction_hash.len(), 0);
    }
    #[test]
    fn test_state_multiple() {
        let mut state = StateStorage::new();
        assert_eq!(state.by_transaction_hash.len(), 0);
        for i in 1..5 {
            let h = crypto_generator::h256();
            state.insert(&h, &3);
            assert_eq!(state.by_transaction_hash.len(), i);
            state.insert(&h, &5);
            assert_eq!(state.by_transaction_hash.len(), i);
        }

    }
    #[test]
    fn test_state_add() {
        let mut state = StateStorage::new();
        assert_eq!(state.by_transaction_hash.len(), 0);
        let tx: Transaction = generator::random_transaction_builder().into();
        let h = tx.hash();
        state.add(&tx);
        assert_eq!(state.by_transaction_hash.len(), 1);
        for i in 0..tx.output.len() {
            assert!(state.contains(&h, &(i as u32)));
        }
        for i in 0..tx.output.len() {
            state.remove(&h, &(i as u32));
        }
        assert_eq!(state.by_transaction_hash.len(), 0);
    }
}