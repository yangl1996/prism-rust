// TODO: Txblock currently has no metadata. It future it could have epsilon.
use crate::crypto::hash::H256;
use std::collections::HashSet;
use std::sync::mpsc::Sender;

/// Message metadata to communicate between blockchain and utxo state
#[derive(PartialEq)]
pub enum UpdateMessage {
    /// Used when new tx blocks are confirmed and the state has to be updated.
    Add,
    /// Used when old tx blocks are UNconfirmed and the state has to be rollbacked.
    Rollback,
}

/// A pool of transaction blocks.
pub struct Pool {
    /// A pool of transaction blocks which are not in the ledger (unconfirmed).
    pub not_in_ledger: HashSet<H256>,
    /// The ordered sequence of confirmed transaction blocks.
    pub ledger: Vec<H256>,
    /// The start index of the blocks confirmed by the leader proposer block at each level.
    pub confirmation_boundary: Vec<usize>,
    /// A pool of unreferred transaction blocks. This is for mining.
    pub unreferred: HashSet<H256>,
    /// Channel to update the utxo state
    pub utxo_update: Sender<(UpdateMessage, Vec<H256>)>,
}

impl Pool {
    /// Create a new transaction block pool.
    pub fn new(utxo_update: Sender<(UpdateMessage, Vec<H256>)>) -> Self {
        let not_in_ledger: HashSet<H256> = HashSet::new();
        let ledger: Vec<H256> = vec![];
        let unreferred: HashSet<H256> = HashSet::new();
        let confirmation_boundary: Vec<usize> = vec![];
        return Self {
            not_in_ledger,
            ledger,
            confirmation_boundary,
            unreferred,
            utxo_update,
        };
    }

    /// Insert a new block to the list of unconfirmed blocks.
    pub fn insert_not_in_ledger(&mut self, hash: H256) {
        self.not_in_ledger.insert(hash);
    }

    /// Check whether a transaction block is in the ledger (i.e. confirmed).
    pub fn is_in_ledger(&self, hash: &H256) -> bool {
        return !self.not_in_ledger.contains(hash);
    }

    /// Mark the confirmation boundary of the given proposer level.
    pub fn mark_confirmation_boundary(&mut self, level: u32) {
        if self.confirmation_boundary.len() + 1 != level as usize {
            panic!("Trying to set the confirmation boundary of a level that has been set, or whose previous level has not been set");
        }
        self.confirmation_boundary.push(self.ledger.len());
    }

    /// Add a transaction block to the ordered ledger, and remove it from the unconfirmed set.
    pub fn add_to_ledger(&mut self, to_add_tx_blocks: Vec<H256>) {
        for tx_block in to_add_tx_blocks.iter() {
            self.ledger.push(*tx_block);
            self.not_in_ledger.remove(tx_block);
        }
        // Ask the utxo state thread to extend its state based on to_add_tx_blocks
        self.utxo_update
            .send((UpdateMessage::Add, to_add_tx_blocks))
            .unwrap();
    }

    /// Roll back the transaction blocks in the ledger confirmed by the leader proposer blocks at
    /// the given level and beyond.
    pub fn rollback_ledger(&mut self, level: usize) {
        // Get the start index of transaction blocks confirmed by leader block at 'level'
        let rollback_start = self.confirmation_boundary[level];
        // Move the tx blocks from the ledger to the unconfirmed set.
        let to_remove_tx_blocks: Vec<H256> = self.ledger.split_off(rollback_start);
        for tx_block in to_remove_tx_blocks.iter() {
            self.insert_not_in_ledger(*tx_block);
        }
        // Ask the utxo state thread to rollback its state for the 'to_remove_tx_blocks'
        self.utxo_update
            .send((UpdateMessage::Rollback, to_remove_tx_blocks))
            .unwrap();

        // Drain confirmation_boundary vector
        self.confirmation_boundary.drain(level - 1..); // TODO: why -1?
    }

    /// Insert a block to the unreferred transaction block set.
    pub fn insert_unreferred(&mut self, hash: H256) {
        self.unreferred.insert(hash);
    }

    /// Remove a block from the unreferred transaction block set.
    pub fn remove_unreferred(&mut self, hash: &H256) {
        self.unreferred.remove(hash);
    }
}
