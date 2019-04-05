#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate clap;

pub mod crypto;
pub mod transaction;
pub mod network;
pub mod block;
pub mod state;
pub mod blockchain;
pub mod miner;
pub mod validation;
pub mod config;
pub mod blockdb;
pub mod wallet;
pub mod handler;

use blockdb::BlockDatabase;
use blockchain::BlockChain;
use miner::memory_pool::MemoryPool;
use std::sync::{Mutex, mpsc, Arc};

pub fn start(addr: std::net::SocketAddr, blockdb: &Arc<BlockDatabase>, 
             blockchain: &Arc<Mutex<BlockChain>>,
             mempool: &Arc<Mutex<MemoryPool>>) -> std::io::Result<(network::server::Handle, miner::miner::Handle)> {
    // create channels between server and worker, worker and miner, miner and worker
    let (msg_sink, msg_source) = mpsc::channel();
    let (ctx_update_sink, ctx_update_source) = mpsc::channel();
    let (mined_block_sink, mined_block_source) = mpsc::channel();

    let (ctx, server) = network::server::new(addr, msg_sink)?;
    ctx.start();

    let ctx = network::worker::new(4, msg_source, blockchain, blockdb, mempool, ctx_update_sink);
    ctx.start();

    let (ctx, miner) = miner::miner::new(mempool, blockchain, blockdb, mined_block_sink, ctx_update_source);
    ctx.start();

    return Ok((server, miner));
}
