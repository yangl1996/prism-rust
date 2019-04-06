// todo: Txblock currently has no metadata. It future it could have epsilon.
use crate::crypto::hash::H256;
use std::collections::HashSet;

pub struct Pool {
    /// Pool of tx blocks which are not in ledger (i.e unconfirmed)
    pub not_in_ledger: HashSet<H256>,
    /// The ledger: Ordered transaction blocks
    pub ledger: Vec<H256>, //
    /// Pool of unreferred tx blocks. For mining
    pub unreferred: HashSet<H256>,
}

impl Pool {
    pub fn new() -> Self {
        let not_in_ledger: HashSet<H256> = HashSet::new();
        let ledger: Vec<H256> = vec![];
        let unreferred: HashSet<H256> = HashSet::new();
        return Self {
            not_in_ledger,
            ledger,
            unreferred,
        };
    }

    /// Adds the block as unconfirmed.
    pub fn insert_not_in_ledger(&mut self, hash: H256) {
        self.not_in_ledger.insert(hash);
    }

    pub fn is_in_ledger(&self, hash: &H256) -> bool {
        return !self.not_in_ledger.contains(hash);
    }

    /// Confirms a tx block by ordering it and removing it from the
    pub fn add_to_ledger(&mut self, hash: &H256) {
        self.ledger.push(*hash); // Add to ordered list
        self.not_in_ledger.remove(hash); // Remove
    }

    pub fn insert_unreferred(&mut self, hash: H256) {
        self.unreferred.insert(hash);
    }

    pub fn remove_unreferred(&mut self, hash: &H256) {
        self.unreferred.remove(hash);
    }
}
