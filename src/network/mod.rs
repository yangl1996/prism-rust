pub mod message;
pub mod server;
pub mod peer;
pub mod worker;

use crate::blockdb::BlockDatabase;
use crate::blockchain::BlockChain;
use crate::miner::memory_pool::MemoryPool;
use crate::config;
use std::sync::{Mutex, mpsc, Arc};

pub fn start(addr: std::net::SocketAddr, blockdb: &Arc<BlockDatabase>, 
             blockchain: &Arc<Mutex<BlockChain>>,
             mempool: &Arc<Mutex<MemoryPool>>) -> std::io::Result<server::Handle> {
    let (msg_sink, msg_source) = mpsc::channel();
    let (ctx, server) = server::new(addr, msg_sink)?;
    ctx.start();

    let ctx = worker::new(4, msg_source, blockchain, blockdb, mempool);
    ctx.start();

    return Ok(server);
}
