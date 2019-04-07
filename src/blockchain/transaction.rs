// todo: Txblock currently has no metadata. It future it could have epsilon.
use crate::crypto::hash::H256;
use std::collections::HashSet;

pub struct Pool {
    /// Pool of tx blocks which are not in ledger (i.e unconfirmed)
    pub not_in_ledger: HashSet<H256>,
    /// The ledger: Ordered transaction blocks
    pub ledger: Vec<H256>,
    /// confirmation_boundaries(l)  is the start index of the blocks confirmed by leader block at level l
    pub confirmation_boundary: Vec<usize>,
    /// Pool of unreferred tx blocks. For mining
    pub unreferred: HashSet<H256>,
}

impl Pool {
    pub fn new() -> Self {
        let not_in_ledger: HashSet<H256> = HashSet::new();
        let ledger: Vec<H256> = vec![];
        let unreferred: HashSet<H256> = HashSet::new();
        let confirmation_boundaries: Vec<usize> = vec![];
        return Self {
            not_in_ledger,
            ledger,
            confirmation_boundary: confirmation_boundaries,
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

    pub fn mark_confirmation_boundary(&mut self, level: u32) {
        println!("Marked conf at level {}", level);
        if self.confirmation_boundary.len() + 1 != level as usize {
            panic!("The proposer level is either already confirmed or its previous level is unconfirmed.\
            Level {}, confirmation_boundary.len(): {}", level, self.confirmation_boundary.len());
        }
        self.confirmation_boundary.push(self.ledger.len());
    }

    /// Confirms a tx block by ordering it and removing it from the
    pub fn add_to_ledger(&mut self, hash: &H256) {
        self.ledger.push(*hash); // Add to ordered list
        self.not_in_ledger.remove(hash); // Remove
    }

    /// Rollback the ledger. Technically, this event occurs only under 51% attack
    pub fn rollback_ledger(&mut self, level: usize) {
        println!("51% attack!!");
        // The start index of tx blocks confirmed by leader block at 'level'
        let rollback_start = self.confirmation_boundary[level];
        // Move the tx blocks in self.ledger confirmed by leader blocks from level onwards to self.not_in_ledger
        let mut to_remove: Vec<H256> = self.ledger.split_off(rollback_start);
        for tx_block in to_remove {
            self.insert_not_in_ledger(tx_block);
        }
    }

    pub fn insert_unreferred(&mut self, hash: H256) {
        self.unreferred.insert(hash);
    }

    pub fn remove_unreferred(&mut self, hash: &H256) {
        self.unreferred.remove(hash);
    }
}
