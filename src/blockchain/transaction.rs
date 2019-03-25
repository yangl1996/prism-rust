// todo: Txblock currently has no metadata. It future it could have epsilon.
use std::collections::HashSet;
use crate::crypto::hash::{H256};


pub struct Pool{
    /// List of unconfirmed tx blocks
    pub unconfirmed: HashSet<H256>,
    /// Ordered transaction blocks
    pub ordered: Vec<H256>, // A confirmed tx block is always ordered for slow confirmation
    /// List of unreferred tx blocks. For mining
    pub unreferred: HashSet<H256>
}

impl Pool {
    pub fn new() -> Self {
        let unconfirmed: HashSet<H256> = HashSet::new();
        let ordered: Vec<H256> = vec![];
        let unreferred: HashSet<H256> = HashSet::new();
        return Self{ unconfirmed, ordered, unreferred};
    }

    /// Adds the block as unconfirmed.
    pub fn insert_unconfirmed(&mut self, hash: H256){
        self.unconfirmed.insert(hash);
    }

    pub fn is_unconfirmed(&self, hash: &H256) -> bool {
        return self.unconfirmed.contains(hash);
    }
    /// Confirms a tx block by ordering it and removing it from the
    pub fn confirm(&mut self, hash: &H256){
        self.ordered.push(*hash); // Order
        self.unconfirmed.remove(hash); // Remove
    }

    pub fn insert_unreferred(&mut self, hash: H256) {
        self.unreferred.insert(hash);
    }

    pub fn remove_unreferred(&mut self, hash: &H256) {
        self.unreferred.remove(hash);
    }
}