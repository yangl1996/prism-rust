use super::message::{self, Message};
use super::peer;
use super::buffer::BlockBuffer;
use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::handler::new_transaction;
use crate::handler::new_validated_block;
use crate::miner::memory_pool::MemoryPool;
use crate::miner::ContextUpdateSignal;
use crate::network::server::Handle as ServerHandle;
use crate::utxodb::UtxoDatabase;
use crate::validation::{check_block, BlockResult};
use crate::wallet::Wallet;
use log::{debug, info};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::collections::HashSet;

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
    buffer: Arc<BlockBuffer>,
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
        buffer: Arc::new(BlockBuffer::new()),
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
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewTransactionHashes(hashes) => {
                    debug!("Got {} new transaction hashes", hashes.len());
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
                    debug!("Asked for {} transactions", hashes.len());
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
                    debug!("Got {} transactions", transactions.len());
                    for transaction in transactions {
                        new_transaction(transaction, &self.mempool, &self.server);
                    }
                }
                Message::NewBlockHashes(hashes) => {
                    debug!("Got {} new block hashes", hashes.len());
                    let mut hashes_to_request = vec![];
                    for hash in hashes {
                        // we need to check blockchain as well
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
                    debug!("Asked for {} blocks", hashes.len());
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
                    debug!("Got {} blocks", blocks.len());
                    let mut to_process: Vec<Block> = blocks;
                    let mut to_request: Vec<H256> = vec![];
                    while let Some(block) = to_process.pop() {
                        let validation_result =
                            check_block(&block, &self.chain, &self.blockdb, &self.utxodb);
                        match validation_result {
                            BlockResult::MissingParent(p) => {
                                debug!("Missing parent block for block {:.8}", block.hash());
                                self.buffer.insert(block, &vec![p]);
                                to_request.push(p);
                            }
                            BlockResult::MissingReferences(r) => {
                                debug!(
                                    "Missing {} referred blocks for block {:.8}",
                                    r.len(),
                                    block.hash()
                                );
                                self.buffer.insert(block, &r);
                                to_request.extend_from_slice(&r);
                            }
                            BlockResult::Pass => {
                                debug!("Processing block {:.8}", block.hash());
                                new_validated_block(
                                    &block,
                                    &self.mempool,
                                    &self.blockdb,
                                    &self.chain,
                                    &self.server,
                                    &self.utxodb,
                                    &self.wallet,
                                );
                                let mut resolved_by_current = self.buffer.satisfy(block.hash());
                                if !resolved_by_current.is_empty() {
                                    debug!("Resolved dependency for {} buffered blocks", resolved_by_current.len());
                                }
                                to_process.append(&mut resolved_by_current);
                            }
                            _ => {
                                debug!(
                                    "Ignoring invalid block {:.8}: {}",
                                    block.hash(),
                                    validation_result
                                );
                                // pass invalid block
                            }
                        }
                    }
                    if !to_request.is_empty() {
                        to_request.sort();
                        to_request.dedup();
                        self.server.broadcast(Message::GetBlocks(to_request));
                    }

                    // tell the miner to update the context
                    self.context_update_chan
                        .send(ContextUpdateSignal::NewContent)
                        .unwrap();
                }
                Message::Bootstrap(after) => {
                    debug!("Asked for all blocks after {}", &after);
                    for batch in self.blockdb.blocks_after(&after, 500) {
                        peer.write(Message::Blocks(batch));
                    }
                }
            }
        }
    }
}
