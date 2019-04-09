use crate::blockdb::BlockDatabase;
use crate::transaction::Transaction;
use std::sync::Mutex;

use super::crypto::hash::{Hashable, H256};
use super::state::{UTXODatabase};

pub fn new_tx_blocks_confirmed(tx_blocks: Vec<H256>, db: &BlockDatabase, state: &Mutex<UTXODatabase>) {

}

pub fn old_tx_blocks_unconfirmed(tx_blocks: Vec<H256>, db: &BlockDatabase, state: &Mutex<UTXODatabase>){

}
