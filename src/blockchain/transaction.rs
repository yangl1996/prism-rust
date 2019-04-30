// TODO: Txblock currently has no metadata. It future it could have epsilon.
use super::database::{BlockChainDatabase, LEDGER_CF};
use crate::crypto::hash::H256;
use bincode::{deserialize, serialize};
use rocksdb::WriteBatch;
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

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
    pub fn new(
        db: Arc<Mutex<BlockChainDatabase>>,
        utxo_update: Sender<(UpdateMessage, Vec<H256>)>,
    ) -> Self {
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

    /// Insert a block to the unreferred transaction block set.
    pub fn insert_unreferred(&mut self, hash: H256) {
        self.unreferred.insert(hash);
    }

    /// Remove a block from the unreferred transaction block set.
    pub fn remove_unreferred(&mut self, hash: &H256) {
        self.unreferred.remove(hash);
    }

    /// Adds transactions block to the ledger (stored in database) which are confirmed by a leader block at level 'level'.
    pub fn add_to_ledger(&mut self, level: u32, to_add_tx_blocks: Vec<H256>) {
        if level <= self.last_prop_confirmed_level {
            panic!("Tx blocks added for level {}", level);
        }
        let db = self.db.lock().unwrap();
        let key = serialize(&level).unwrap();
        let value = serialize(&to_add_tx_blocks).unwrap();
        let cf = db.handle.cf_handle(LEDGER_CF).unwrap();
        match db.handle.put_cf(cf, &key, &value) {
            Ok(_) => {
                for tx_block in to_add_tx_blocks.iter() {
                    self.not_in_ledger.remove(tx_block);
                }
                self.last_prop_confirmed_level = level;
                self.utxo_update
                    .send((UpdateMessage::Add, to_add_tx_blocks))
                    .unwrap();
            }
            Err(e) => {
                panic!("Tx blocks not added to the ledger in the db,  Error {}", e);
            }
        }
    }

    /// Roll back the transaction blocks in the ledger confirmed by the leader proposer blocks at
    /// the given level and beyond.
    pub fn rollback_ledger(&mut self, rollback_start_level: u32) {

        // 1. Get all the tx blocks confirmed between levels =rollback_start_level and self.last_prop_confirmed_level
        let mut removed_tx_blocks: Vec<H256> = vec![]; //stores the tx blocks which are removed from the ledger due  to rollback
        for level in rollback_start_level..self.last_prop_confirmed_level {
            let key = serialize(&level).unwrap();
            let confirmed_blocks_level = self.get_blocks_at_level(level);
            removed_tx_blocks.extend(confirmed_blocks_level);
        }

        //2. Atomic delete tx blocks from level rollback_start_level to self.last_prop_confirmed_level.
        let db = self.db.lock().unwrap();
        let cf = db.handle.cf_handle(LEDGER_CF).unwrap();
        let mut batch = WriteBatch::default();
        for level in rollback_start_level..self.last_prop_confirmed_level {
            let key = serialize(&level).unwrap();
            batch.delete_cf(cf, &key);
        }
        match db.handle.write(batch) {
            Ok(_) => {
                // If the blocks are deleted,
                drop(db);
                // 2b.1
                self.last_prop_confirmed_level = rollback_start_level - 1;
                // 2b.2 Add the removed tx blocks back to the unconfirmed set.
                for tx_block in removed_tx_blocks.iter() {
                    self.insert_not_in_ledger(*tx_block);
                }
                //2b.3 Ask the utxo state thread to rollback its state for the 'to_remove_tx_blocks'
                self.utxo_update
                    .send((UpdateMessage::Rollback, removed_tx_blocks))
                    .unwrap();
            }
            Err(e) => {
                panic!("DB error {}", e);
            }
        }
    }

    pub fn get_ledger(&mut self) ->Vec<H256> {
        let mut  ledger: Vec<H256> = vec![];
        for level in 1..=self.last_prop_confirmed_level {
            let confirmed_blocks_level = self.get_blocks_at_level(level);
            ledger.extend(confirmed_blocks_level);
        }
        return ledger;
    }

    // Returns the tx blocks confirmed because of a leader block at level 'level'
    fn get_blocks_at_level(&mut self, level: u32) -> Vec<H256> {
        let db = self.db.lock().unwrap();
        let key = serialize(&level).unwrap();
        let cf = db.handle.cf_handle(LEDGER_CF).unwrap();
        let serialized_result = db.handle.get_cf(cf, &key);
        match serialized_result {
            Err(e) => panic!("Database error"),
            Ok(serialized_option) => match serialized_option {
                None => panic!("Node data not present at level {}", level),
                Some(s) => return deserialize(&s).unwrap(),
            },
        }
    }


}
