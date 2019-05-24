#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate lazy_static;

pub mod block;
pub mod blockchain;
pub mod blockdb;
pub mod config;
pub mod crypto;
pub mod handler;
pub mod miner;
pub mod network;
pub mod transaction;
pub mod utxodb;
pub mod validation;
pub mod visualization;
pub mod wallet;
pub mod api;
pub mod experiment;

use crate::utxodb::UtxoDatabase;
use bincode::serialize;
use blockchain::BlockChain;
use blockdb::BlockDatabase;
use crypto::hash::Hashable;
use crypto::sign::PubKey;
use miner::memory_pool::MemoryPool;
use std::sync::{mpsc, Arc, Mutex};
use transaction::{CoinId, Input, Output, Transaction};
use wallet::Wallet;

/// Gives 100 coins of 100 worth to every given address.
pub fn ico(
    recipients: &[crypto::hash::H256], // addresses of all the ico recipients
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
) -> Result<(), rocksdb::Error> {
    let funding = Transaction {
        input: vec![],
        output: recipients
            .iter()
            .map(|recipient| {
                (0..100).map(move |_| Output {
                    value: 100,
                    recipient: recipient.clone(),
                })
            })
            .flatten()
            .collect(),
        authorization: vec![],
    };
    let diff = utxodb.apply_diff(&[funding], &[]).unwrap();
    wallet.apply_diff(&diff.0, &diff.1).unwrap();
    Ok(())
}
