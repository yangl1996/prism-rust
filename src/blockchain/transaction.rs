// TODO: Txblock currently has no metadata. It future it could have epsilon.
use crate::crypto::hash::H256;
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use super::database::{BlockChainDatabase, LEDGER_CF};
use bincode::{deserialize, serialize};

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
    pub db: Arc<Mutex<BlockChainDatabase>>,
    /// The last level of prop block which is confirmed
    pub last_prop_confirmed_level: u32,
    /// A pool of unconfirmed transaction blocks which are not in the ledger.
    pub not_in_ledger: HashSet<H256>,
    /// A pool of unreferred transaction blocks. This is for mining.
    pub unreferred: HashSet<H256>,
    /// Channel to update the utxo state
    pub utxo_update: Sender<(UpdateMessage, Vec<H256>)>,
}

impl Pool {
    /// Create a new transaction block pool.
    pub fn new(db: Arc<Mutex<BlockChainDatabase>>, utxo_update: Sender<(UpdateMessage, Vec<H256>)>) -> Self {
        let not_in_ledger: HashSet<H256> = HashSet::new();
        let ledger: Vec<H256> = vec![];
        let unreferred: HashSet<H256> = HashSet::new();
        let last_prop_confirmed_level: u32 = 0;
        return Self {
            db,
            last_prop_confirmed_level,
            not_in_ledger,
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
    pub fn update_last_prop_confirmed_level(&mut self, level: u32) {
        if self.last_prop_confirmed_level = level;
    }

    /// Adds transactions block to the database cf which are confirmed by a leader block at level 'level'.
    pub fn add_to_ledger(&mut self, level: u32, to_add_tx_blocks: Vec<H256>) {
        let key = serialize(&level).unwrap();
        let value = serialize(&to_add_tx_blocks).unwrap();
        let cf = self.handle.cf_handle(LEDGER_CF).unwrap();
        match self.handle.put(cf, &key, &value) {
            Ok(_) => {
                for tx_block in to_add_tx_blocks.iter() {
                    self.not_in_ledger.remove(tx_block);
                }
                self.utxo_update
                    .send((UpdateMessage::Add, to_add_tx_blocks))
                    .unwrap();
            }
            Err(e) => {
                panic!("Tx blocks not added to the ledger in the db");
            }
        }



    }

    /// Roll back the transaction blocks in the ledger confirmed by the leader proposer blocks at
    /// the given level and beyond.
    pub fn rollback_ledger(&mut self, rollback_start_level: usize) {
        let cf = self.handle.cf_handle(LEDGER_CF).unwrap();
        for level in rollback_start_level..self.last_prop_confirmed_level{
            let key = serialize(&level).unwrap();

            self.handle.delete_cf(cf, &key);
        }
        self.last_prop_confirmed_level = rollback_start_level - 1;
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
