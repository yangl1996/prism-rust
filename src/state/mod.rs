use crate::crypto::hash::{H256, Hashable};
use std::collections::{HashSet, HashMap};
use crate::transaction::{Transaction, Input, Output};

// TODO: learn from Parity

#[derive(Debug)]
pub struct Storage {
    /// Transaction outpoint -> coin value and owner.
    by_outpoint: HashMap<Input, Output>,
}

impl Storage {
    pub fn new() -> Self {
        return Self {
            by_outpoint: HashMap::new(),
        };
    }

    pub fn insert(&mut self, outpoint: Input, coin: Output) {
        self.by_outpoint.insert(outpoint, coin);
        return;
    }

    pub fn contains(&self, outpoint: &Input) -> bool {
        return self.by_outpoint.contains_key(outpoint);
    }

    pub fn remove(&mut self, outpoint: &Input) {
        self.by_outpoint.remove(outpoint);
        return;
    }
}

// TODO: add tests.
/*
#[cfg(test)]
pub mod tests {
    use super::Storage;
    use crate::crypto;
    use crate::transaction::{Transaction, generator};
    use crate::crypto::hash::Hashable;

    #[test]
    fn test_state_basic() {
        let mut state = StateStorage::new();
        assert_eq!(state.by_outpoint.len(), 0);
        let h = crypto::generator::h256();
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
*/
