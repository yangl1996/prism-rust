use super::message::{self, Message};
use super::peer;
use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::handler::new_validated_block;
use crate::handler::new_transaction;
use crate::miner::memory_pool::MemoryPool;
use crate::miner::ContextUpdateSignal;
use crate::network::server::Handle as ServerHandle;
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use crate::validation::{check_block, BlockResult};
use log::{debug, info};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct Context {
    msg_chan: Arc<Mutex<mpsc::Receiver<(message::Message, peer::Handle)>>>,
    num_worker: usize,
    chain: Arc<BlockChain>,
    blockdb: Arc<BlockDatabase>,
    utxodb: Arc<UtxoDatabase>,
    wallet: Arc<Wallet>,
    mempool: Arc<Mutex<MemoryPool>>,
    context_update_chan: mpsc::Sender<ContextUpdateSignal>,
    server: ServerHandle,
    buffer: Arc<Mutex<Vec<Block>>>,
}

pub fn new(
    num_worker: usize,
    msg_src: mpsc::Receiver<(message::Message, peer::Handle)>,
    blockchain: &Arc<BlockChain>,
    blockdb: &Arc<BlockDatabase>,
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
    mempool: &Arc<Mutex<MemoryPool>>,
    ctx_update_sink: mpsc::Sender<ContextUpdateSignal>,
    server: ServerHandle,
) -> Context {
    let ctx = Context {
        msg_chan: Arc::new(Mutex::new(msg_src)),
        num_worker: num_worker,
        chain: Arc::clone(blockchain),
        blockdb: Arc::clone(blockdb),
        utxodb: Arc::clone(utxodb),
        wallet: Arc::clone(wallet),
        mempool: Arc::clone(mempool),
        context_update_chan: ctx_update_sink,
        server: server,
        buffer: Arc::new(Mutex::new(vec![])),
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
                Message::NewTransactionHashes(hashes) => {
                    debug!("NewTransactionHashes");
                    let mut hashes_to_request = vec![];
                    for hash in hashes {
                        if !self.mempool.lock().unwrap().contains(&hash) {
                            hashes_to_request.push(hash);
                        }
                    }
                    if hashes_to_request.len() != 0 {
                        peer.write(Message::GetTransactions(hashes_to_request));
                    }
                }
                Message::GetTransactions(hashes) => {
                    debug!("GetTransactions");
                    let mut transactions = vec![];
                    for hash in hashes {
                        match self.mempool.lock().unwrap().get(&hash) {
                            None => {}
                            Some(entry) => {
                                transactions.push(entry.transaction.clone());
                            }
                        }
                    }
                    peer.write(Message::Transactions(transactions));
                }
                Message::Transactions(transactions) => {
                    debug!("Transactions");
                    for transaction in transactions {
                        new_transaction(transaction, &self.mempool, &self.server);
                    }
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
                    for block in blocks {
                        // TODO: add validation and buffer logic here
                        let validation_result =
                            check_block(&block, &self.chain, &self.blockdb, &self.utxodb);
                        match validation_result {
                            BlockResult::MissingParent(p) => {
                                debug!("Missing parent block");
                                self.buffer.lock().unwrap().push(block);
                                self.server.broadcast(Message::GetBlocks(vec![p]));
                            }
                            BlockResult::MissingReferences(r) => {
                                debug!("Missing referred block");
                                self.buffer.lock().unwrap().push(block);
                                self.server.broadcast(Message::GetBlocks(r));
                            }
                            BlockResult::Pass => {
                                debug!("Adding new block");
                                new_validated_block(
                                    &block,
                                    &self.mempool,
                                    &self.blockdb,
                                    &self.chain,
                                    &self.server,
                                    &self.utxodb,
                                    &self.wallet,
                                );
                            }
                            _ => {
                                debug!("Invalid block: {}", validation_result);
                                // pass invalid block
                            }
                        }
                    }

                    let mut buffer = self.buffer.lock().unwrap();
                    buffer.retain(|block| {
                        let validation_result =
                            check_block(&block, &self.chain, &self.blockdb, &self.utxodb);
                        match validation_result {
                            BlockResult::MissingParent(_) | BlockResult::MissingReferences(_) => {
                                return true;    // retain this block
                            }
                            BlockResult::Pass => {
                                // TODO: avoid inserting the same block again here
                                new_validated_block(
                                    &block,
                                    &self.mempool,
                                    &self.blockdb,
                                    &self.chain,
                                    &self.server,
                                    &self.utxodb,
                                    &self.wallet,
                                );
                                return false;   // remove the block
                            }
                            _ => {
                                return false;   // remove invalid block
                            }
                        }
                    });
                    drop(buffer);

                    // tell the miner to update the context
                    self.context_update_chan
                        .send(ContextUpdateSignal::NewContent)
                        .unwrap();
                }
            }
        }
    }
}
