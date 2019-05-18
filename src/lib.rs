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

use crate::utxodb::UtxoDatabase;
use blockchain::BlockChain;
use blockdb::BlockDatabase;
use miner::memory_pool::MemoryPool;
use std::sync::{mpsc, Arc, Mutex};
use wallet::Wallet;
use crypto::sign::PubKey;
use crypto::hash::Hashable;
use transaction::{CoinId, Input, Output, Transaction};
use bincode::serialize;

pub fn start(
    addr: std::net::SocketAddr,
    blockdb: &Arc<BlockDatabase>,
    utxodb: &Arc<UtxoDatabase>,
    blockchain: &Arc<BlockChain>,
    wallet: &Arc<Wallet>,
    mempool: &Arc<Mutex<MemoryPool>>,
) -> std::io::Result<(network::server::Handle, miner::Handle)> {
    // create channels between server and worker, worker and miner, miner and worker
    let (msg_tx, msg_rx) = mpsc::channel();
    let (ctx_tx, ctx_rx) = mpsc::channel();
    let ctx_tx_wallet = ctx_tx.clone();

    let (ctx, server) = network::server::new(addr, msg_tx)?;
    ctx.start().unwrap();

    let ctx = network::worker::new(
        4,
        msg_rx,
        blockchain,
        blockdb,
        utxodb,
        wallet,
        mempool,
        ctx_tx,
        server.clone(),
    );
    ctx.start();

    let (ctx, miner) = miner::new(
        mempool,
        blockchain,
        utxodb,
        wallet,
        blockdb,
        ctx_rx,
        server.clone(),
    );
    ctx.start();

    // TODO: all wallet-related logic are just for demoing. We need an API for user to send/receive
    // money. For now, we just initialize the wallet here and let it send transactions to itself
    // periodically.
    wallet.generate_keypair().unwrap();

    return Ok((server, miner));
}


/// Gives 100 coins of 100 worth to every public key.
pub fn ico(
    pub_keys: Vec<PubKey>, // public keys of all the ico recipients
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
) -> Result<(), rocksdb::Error> {
    let funding = Transaction {
        input: vec![],
        output: pub_keys
            .iter()
            .map(|pub_key| {
                (0..100).map(move |_| Output {
                    value: 100,
                    recipient: pub_key.hash().clone(),
                })
            })
            .flatten()
            .collect(),
        authorization: vec![],
    };
    let mut funding_coins: Vec<Input> = vec![];
    let transaction_hash = funding.hash();
    for (idx, output) in funding.output.iter().enumerate() {
        let id = CoinId {
            hash: transaction_hash,
            index: idx as u32,
        };
        utxodb.db
            .put(serialize(&id).unwrap(), serialize(&output).unwrap())?;
        let coin = Input {
            coin: id,
            value: output.value,
            owner: output.recipient,
        };
        funding_coins.push(coin);
    }
    wallet.update(&funding_coins, &vec![]).unwrap();
    Ok(())
}
