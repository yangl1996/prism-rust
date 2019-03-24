// todo: Txblock currently has no metadata. It future it could have epsilon.
use std::collections::HashSet;
use crate::crypto::hash::{H256};


pub struct Pool{
    /// List of unconfirmed tx blocks
    pub unconfirmed: HashSet<H256>,
    /// Ordered transaction blocks
    pub ordered: Vec<H256>, // A confirmed tx block is always ordered for slow confirmation
}

impl Pool {
    pub fn new() -> Self {
        let unconfirmed_transaction_blocks: HashSet<H256> = HashSet::new();
        let ordered_transaction_blocks: Vec<H256> = vec![];
        return Self{ unconfirmed: unconfirmed_transaction_blocks, ordered: ordered_transaction_blocks };
    }

    /// Adds the block as unconfirmed.
    pub fn insert_unconfirmed(&mut self, hash: H256){
        self.unconfirmed.insert(hash);
    }

    /// Confirms a tx block by ordering it and removing it from the
    pub fn confirm(&mut self, hash: H256){
        self.ordered.push(hash); // Order
        self.unconfirmed.remove(&hash); // Remove
    }
}