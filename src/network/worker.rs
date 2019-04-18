use super::message::{self, Message};
use super::peer;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::handler::new_validated_block;
use crate::miner::memory_pool::MemoryPool;
use crate::miner::miner::ContextUpdateSignal;
use crate::network::server::Handle as ServerHandle;
use log::{debug, info};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct Context {
    msg_chan: Arc<Mutex<mpsc::Receiver<(message::Message, peer::Handle)>>>,
    num_worker: usize,
    chain: Arc<Mutex<BlockChain>>,
    blockdb: Arc<BlockDatabase>,
    mempool: Arc<Mutex<MemoryPool>>,
    context_update_chan: mpsc::Sender<ContextUpdateSignal>,
    server: ServerHandle,
}

pub fn new(
    num_worker: usize,
    msg_src: mpsc::Receiver<(message::Message, peer::Handle)>,
    blockchain: &Arc<Mutex<BlockChain>>,
    blockdb: &Arc<BlockDatabase>,
    mempool: &Arc<Mutex<MemoryPool>>,
    ctx_update_sink: mpsc::Sender<ContextUpdateSignal>,
    server: ServerHandle,
) -> Context {
    let ctx = Context {
        msg_chan: Arc::new(Mutex::new(msg_src)),
        num_worker: num_worker,
        chain: Arc::clone(blockchain),
        blockdb: Arc::clone(blockdb),
        mempool: Arc::clone(mempool),
        context_update_chan: ctx_update_sink,
        server: server,
    };
    return ctx;
}

impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        for _ in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let chan = self.msg_chan.lock().unwrap();
            let msg = chan.recv().unwrap();
            drop(chan);
            let (msg, peer) = msg;
            match msg {
                Message::Ping(nonce) => {
                    info!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    info!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(hashes) => {
                    debug!("NewBlockHashes");
                    let mut hashes_to_request = vec![];
                    for hash in hashes {
                        // TODO: add a method to blockdb to quickly check whether a block exists
                        match self.blockdb.get(&hash).unwrap() {
                            None => {
                                hashes_to_request.push(hash);
                            }
                            _ => {}
                        }
                    }
                    if hashes_to_request.len() != 0 {
                        peer.write(Message::GetBlocks(hashes_to_request));
                    }
                }
                Message::GetBlocks(hashes) => {
                    debug!("GetBlocks");
                    let mut blocks = vec![];
                    for hash in hashes {
                        match self.blockdb.get(&hash).unwrap() {
                            None => {}
                            Some(block) => {
                                blocks.push(block);
                            }
                        }
                    }
                    peer.write(Message::Blocks(blocks));
                }
                Message::Blocks(blocks) => {
                    debug!("Blocks");
                    // TODO: add validation and buffer logic here
                    for block in blocks {
                        // TODO: avoid inserting the same block again here
                        new_validated_block(
                            block,
                            &self.mempool,
                            &self.blockdb,
                            &self.chain,
                            &self.server,
                        );
                    }
                    // tell the miner to update the context
                    self.context_update_chan
                        .send(ContextUpdateSignal::NewContent)
                        .unwrap();
                }
                _ => {}
            }
        }
    }
}
