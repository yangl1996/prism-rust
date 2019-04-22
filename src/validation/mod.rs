pub mod data_availability;
pub mod header;
pub mod transaction;
pub mod transaction_block;
pub mod proposer_block;
pub mod voter_block;
use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::Transaction;
use std::sync::{Arc, Mutex};

pub enum BlockResult {
    Pass,
    MissingReferences(Vec<H256>),
    Fail,
}
