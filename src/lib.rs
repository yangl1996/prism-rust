#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;

pub mod block;
pub mod blockchain;
pub mod blockdb;
pub mod config;
pub mod crypto;
pub mod handler;
pub mod miner;
pub mod network;
pub mod state;
pub mod transaction;
pub mod validation;
pub mod visualization;
pub mod wallet;

use blockchain::BlockChain;
use blockdb::BlockDatabase;
use miner::memory_pool::MemoryPool;
use std::sync::{mpsc, Arc, Mutex};
use crate::state::UTXODatabase;
use crate::crypto::hash::H256;
use crate::blockchain::transaction::UpdateMessage as LedgerUpdateMessage;

pub fn start(
    addr: std::net::SocketAddr,
    blockdb: &Arc<BlockDatabase>,
    blockchain: &Arc<Mutex<BlockChain>>,
    utxodb: &Arc<Mutex<UTXODatabase>>,
    mempool: &Arc<Mutex<MemoryPool>>,
    state_update_source: mpsc::Receiver<(LedgerUpdateMessage, Vec<H256>)>,
) -> std::io::Result<(
    network::server::Handle,
    miner::miner::Handle,
    Arc<Vec<Mutex<wallet::Wallet>>>,
)> {
    // create channels between server and worker, worker and miner, miner and worker
    let (msg_sink, msg_source) = mpsc::channel();
    let (ctx_update_sink, ctx_update_source) = mpsc::channel();
    let ctx_update_sink_wallet = ctx_update_sink.clone();

    let (ctx, server) = network::server::new(addr, msg_sink)?;
    ctx.start().unwrap();

    let ctx = network::worker::new(
        4,
        msg_source,
        blockchain,
        blockdb,
        mempool,
        ctx_update_sink,
        server.clone(),
    );
    ctx.start();

    let (ctx, miner) = miner::miner::new(
        mempool,
        blockchain,
        blockdb,
        ctx_update_source,
        server.clone(),
    );
    ctx.start();

    let mut wallet = wallet::Wallet::new(mempool, ctx_update_sink_wallet);
    wallet.generate_keypair();
    let wallets = Arc::new(vec![Mutex::new(wallet)]);

    //state_updater part
    let ctx = state::updater::new(
        blockdb,
        utxodb,
        &wallets,
        state_update_source,
    );
    ctx.start();

    return Ok((server, miner, wallets));
}
