pub mod memory_pool;

use crate::block::header::Header;
use crate::block::{proposer, transaction, voter};
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::handler::new_validated_block;
use crate::network::message::Message;
use crate::network::server::Handle as ServerHandle;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use memory_pool::MemoryPool;
use std::time;
use std::time::SystemTime;

use rand::distributions::Distribution;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::thread;

use rand::Rng;

enum ControlSignal {
    Start(u64, bool), // the number controls the lambda of interval between block generation
    Step,
    Exit,
}

#[derive(Ord, Eq, PartialOrd, PartialEq)]
pub enum ContextUpdateSignal {
    // TODO: New transaction comes, we update transaction block's content
    //NewTx,//should be called: mem pool change
    // New proposer block comes, we need to update all contents' parent
    NewProposerBlock,
    // New voter block comes, we need to update that voter chain
    NewVoterBlock(u16),
    // New transaction block comes, we need to update proposer content's tx ref
    NewTransactionBlock,
}

enum OperatingState {
    Paused,
    Run(u64, bool),
    Step,
    ShutDown,
}

pub struct Context {
    blockdb: Arc<BlockDatabase>,
    blockchain: Arc<BlockChain>,
    mempool: Arc<Mutex<MemoryPool>>,
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    /// Channel for notifying miner of new content
    context_update_chan: Receiver<ContextUpdateSignal>,
    context_update_tx: Sender<ContextUpdateSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    header: Header,
    contents: Vec<Content>,
    content_merkle_tree: MerkleTree,
    config: BlockchainConfig,
}

#[derive(Clone)]
pub struct Handle {
    // Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    mempool: &Arc<Mutex<MemoryPool>>,
    blockchain: &Arc<BlockChain>,
    blockdb: &Arc<BlockDatabase>,
    ctx_update_source: Receiver<ContextUpdateSignal>,
    ctx_update_tx: &Sender<ContextUpdateSignal>,
    server: &ServerHandle,
    config: BlockchainConfig,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let mut contents: Vec<Content> = vec![];

    let proposer_content = proposer::Content {
        transaction_refs: vec![],
        proposer_refs: vec![],
    };
    contents.push(Content::Proposer(proposer_content));

    let transaction_content = transaction::Content {
        transactions: vec![],
    };
    contents.push(Content::Transaction(transaction_content));

    for voter_idx in 0..config.voter_chains {
        let content = voter::Content {
            chain_number: voter_idx as u16,
            voter_parent: config.voter_genesis[voter_idx as usize],
            votes: vec![],
        };
        contents.push(Content::Voter(content));
    }

    let content_merkle_tree = MerkleTree::new(&contents);

    let ctx = Context {
        blockdb: Arc::clone(blockdb),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool),
        control_chan: signal_chan_receiver,
        context_update_chan: ctx_update_source,
        context_update_tx: ctx_update_tx.clone(),
        operating_state: OperatingState::Paused,
        server: server.clone(),
        header: Header {
            parent: config.proposer_genesis,
            timestamp: get_time(),
            nonce: 0,
            content_merkle_root: H256::default(),
            extra_content: [0; 32],
            difficulty: *DEFAULT_DIFFICULTY,
        },
        contents,
        content_merkle_tree,
        config,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64, lazy: bool) {
        self.control_chan
            .send(ControlSignal::Start(lambda, lazy))
            .unwrap();
    }

    pub fn step(&self) {
        self.control_chan.send(ControlSignal::Step).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i, l) => {
                info!(
                    "Miner starting in continuous mode with lambda {} and lazy mode {}",
                    i, l
                );
                self.operating_state = OperatingState::Run(i, l);
            }
            ControlSignal::Step => {
                info!("Miner starting in stepping mode");
                self.operating_state = OperatingState::Step;
            }
        }
    }

    fn miner_loop(&mut self) {
        // tell ourself to update all context
        self.context_update_tx
            .send(ContextUpdateSignal::NewProposerBlock);
        self.context_update_tx
            .send(ContextUpdateSignal::NewTransactionBlock);
        for voter_chain in 0..self.config.voter_chains {
            self.context_update_tx
                .send(ContextUpdateSignal::NewVoterBlock(voter_chain as u16));
        }

        let mut rng = rand::thread_rng();

        // main mining loop
        loop {
            let block_start = time::Instant::now();

            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // check whether there is new content through context update channel
            let mut new_transaction_block: bool = false;
            let mut new_voter_block: BTreeSet<u16> = BTreeSet::new();
            let mut new_proposer_block: bool = false;
            for sig in self.context_update_chan.try_iter() {
                match sig {
                    ContextUpdateSignal::NewProposerBlock => new_proposer_block = true,
                    ContextUpdateSignal::NewVoterBlock(chain) => {
                        new_voter_block.insert(chain);
                    }
                    ContextUpdateSignal::NewTransactionBlock => new_transaction_block = true,
                }
            }

            // handle context updates
            let mut touched_content: BTreeSet<u16> = BTreeSet::new();
            let mut voter_shift = false;
            // update voter parents
            for voter_chain in new_voter_block.iter() {
                let chain_id: usize = (FIRST_VOTER_INDEX + voter_chain) as usize;
                let voter_parent = self.blockchain.best_voter(*voter_chain as usize);
                if let Content::Voter(c) = &mut self.contents[chain_id] {
                    if &voter_parent != &c.voter_parent {
                        c.voter_parent = voter_parent;
                        // mark that we have shifted a vote
                        voter_shift = true;
                        touched_content.insert(chain_id as u16);
                    }
                } else {
                    unreachable!();
                }
            }

            // update transaction block content
            if new_transaction_block {
                let mempool = self.mempool.lock().unwrap();
                let transactions = mempool.get_transactions(self.config.tx_txs);
                drop(mempool);
                let _chain_id: usize = TRANSACTION_INDEX as usize;
                if let Content::Transaction(c) = &mut self.contents[TRANSACTION_INDEX as usize] {
                    c.transactions = transactions;
                    touched_content.insert(TRANSACTION_INDEX);
                } else {
                    unreachable!();
                }
            }

            // append transaction references
            // FIXME: we are now refreshing the whole tree
            // note that if there are new proposer blocks, we will need to refresh tx refs in the
            // next step. In that case, don't bother doing it here.
            if new_transaction_block && !new_proposer_block {
                if let Content::Proposer(c) = &mut self.contents[PROPOSER_INDEX as usize] {
                    // only update the references if we are not running out of quota
                    if c.transaction_refs.len() < self.config.proposer_tx_refs as usize {
                        let mut refs = self.blockchain.unreferred_transactions();
                        refs.truncate(self.config.proposer_tx_refs as usize);
                        c.transaction_refs = refs;
                        touched_content.insert(PROPOSER_INDEX);
                    }
                } else {
                    unreachable!();
                }
            }

            // update the best proposer
            if new_proposer_block {
                self.header.parent = self.blockchain.best_proposer().unwrap();
            }

            // update the best proposer and the proposer/transaction refs. Note that if the best
            // proposer block is updated, we will update the proposer/transaction refs. But we also
            // need to make sure that the best proposer is still the best at the end of this
            // process. Otherwise, we risk having voter/transaction blocks that have a parent
            // deeper than ours
            // sadly, we still may have race condition where the best proposer is updated, but the
            // blocks it refers to have not been removed from unreferred_{proposer, transaction}.
            // but this is pretty much the only race condition that we still have.
            loop {
                // first refresh the transaction and proposer refs if there has been a new proposer
                // block
                if new_proposer_block {
                    if let Content::Proposer(c) = &mut self.contents[PROPOSER_INDEX as usize] {
                        let mut refs = self.blockchain.unreferred_transactions();
                        refs.truncate(self.config.proposer_tx_refs as usize);
                        c.transaction_refs = refs;
                        c.proposer_refs = self.blockchain.unreferred_proposers();
                        let parent = self.header.parent;
                        c.proposer_refs.retain(|&x| x != parent);
                        touched_content.insert(PROPOSER_INDEX);
                    } else {
                        unreachable!();
                    }
                }

                // then check whether our proposer parent is really the best
                let best_proposer = self.blockchain.best_proposer().unwrap();
                if self.header.parent == best_proposer {
                    break;
                } else {
                    new_proposer_block = true;
                    self.header.parent = best_proposer;
                    continue;
                }
            }

            // update the votes
            if new_proposer_block {
                for voter_chain in 0..self.config.voter_chains {
                    let chain_id: usize = (FIRST_VOTER_INDEX + voter_chain) as usize;
                    let voter_parent = if let Content::Voter(c) = &self.contents[chain_id] {
                        c.voter_parent
                    } else {
                        unreachable!();
                    };
                    if let Content::Voter(c) = &mut self.contents[chain_id] {
                        c.votes = self
                            .blockchain
                            .unvoted_proposer(&voter_parent, &self.header.parent)
                            .unwrap();
                        touched_content.insert(chain_id as u16);
                    } else {
                        unreachable!();
                    }
                }
            } else if !new_voter_block.is_empty() {
                for voter_chain in 0..self.config.voter_chains {
                    let chain_id: usize = (FIRST_VOTER_INDEX + voter_chain) as usize;
                    let voter_parent = if let Content::Voter(c) = &self.contents[chain_id] {
                        c.voter_parent
                    } else {
                        unreachable!();
                    };
                    if let Content::Voter(c) = &mut self.contents[chain_id] {
                        c.votes = self
                            .blockchain
                            .unvoted_proposer(&voter_parent, &self.header.parent)
                            .unwrap();
                        touched_content.insert(chain_id as u16);
                    } else {
                        unreachable!();
                    }
                }
            }

            // update the difficulty
            self.header.difficulty = self.get_difficulty(&self.header.parent);

            // update or rebuild the merkle tree according to what we did in the last stage
            if new_proposer_block || voter_shift {
                // if there has been a new proposer block, simply rebuild the merkle tree
                self.content_merkle_tree = MerkleTree::new(&self.contents);
            } else {
                // if there has not been a new proposer block, update individual entries
                // TODO: add batch updating to merkle tree
                for voter_chain in new_voter_block.iter() {
                    let chain_id = (FIRST_VOTER_INDEX + voter_chain) as usize;
                    self.content_merkle_tree
                        .update(chain_id, &self.contents[chain_id]);
                }
                if new_transaction_block {
                    self.content_merkle_tree.update(
                        TRANSACTION_INDEX as usize,
                        &self.contents[TRANSACTION_INDEX as usize],
                    );
                    if touched_content.contains(&PROPOSER_INDEX) {
                        self.content_merkle_tree.update(
                            PROPOSER_INDEX as usize,
                            &self.contents[PROPOSER_INDEX as usize],
                        );
                    }
                }
            }

            // update merkle root if anything happened in the last stage
            if new_proposer_block || !new_voter_block.is_empty() || new_transaction_block {
                self.header.content_merkle_root = self.content_merkle_tree.root();
            }

            // try a new nonce, and update the timestamp
            self.header.nonce = rng.gen();
            self.header.timestamp = get_time();

            // Check if we successfully mined a block
            let header_hash = self.header.hash();
            if header_hash < self.header.difficulty {
                // Create a block
                let mined_block: Block = self.produce_block(header_hash);
                //if the mined block is an empty tx block, we ignore it, and go straight to next mining loop
                let skip: bool = {
                    if let OperatingState::Run(_, lazy) = self.operating_state {
                        if lazy {
                            match &mined_block.content {
                                Content::Transaction(content) => {
                                    content.transactions.is_empty()
                                }
                                Content::Voter(content) => content.votes.is_empty(),
                                Content::Proposer(content) => {
                                    content.transaction_refs.is_empty()
                                        && content.proposer_refs.is_empty()
                                }
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                if !skip {
                    PERFORMANCE_COUNTER.record_mine_block(&mined_block);
                    self.blockdb.insert(&mined_block).unwrap();
                    new_validated_block(
                        &mined_block,
                        &self.mempool,
                        &self.blockdb,
                        &self.blockchain,
                        &self.server,
                    );
                    // broadcast after adding the new block to the blockchain, in case a peer mines
                    // a block immediately after we broadcast, leaving us non time to insert into
                    // the blockchain
                    self.server
                        .broadcast(Message::NewBlockHashes(vec![header_hash]));
                    // if we are stepping, pause the miner loop
                    if let OperatingState::Step = self.operating_state {
                        self.operating_state = OperatingState::Paused;
                    }
                }
                // after we mined this block, we update the context based on this block
                match &mined_block.content {
                    Content::Proposer(_) => self
                        .context_update_tx
                        .send(ContextUpdateSignal::NewProposerBlock)
                        .unwrap(),
                    Content::Voter(content) => self
                        .context_update_tx
                        .send(ContextUpdateSignal::NewVoterBlock(content.chain_number))
                        .unwrap(),
                    Content::Transaction(_) => self
                        .context_update_tx
                        .send(ContextUpdateSignal::NewTransactionBlock)
                        .unwrap(),
                }
            }

            if let OperatingState::Run(i, _) = self.operating_state {
                if i != 0 {
                    let interval_dist = rand::distributions::Exp::new(1.0 / (i as f64));
                    let interval = interval_dist.sample(&mut rng);
                    let interval = time::Duration::from_micros(interval as u64);
                    let time_spent = time::Instant::now().duration_since(block_start);
                    if interval > time_spent {
                        thread::sleep(interval - time_spent);
                    }
                }
            }
        }
    }

    /// Given a valid header, sortition its hash and create the block
    fn produce_block(&self, header_hash: H256) -> Block {
        // Get sortition ID
        let sortition_id = self
            .config
            .sortition_hash(&header_hash, &self.header.difficulty)
            .expect("Block Hash should <= Difficulty");
        // Create a block
        // get the merkle proof
        let sortition_proof: Vec<H256> = self.content_merkle_tree.proof(sortition_id as usize);
        Block::from_header(
            self.header,
            self.contents[sortition_id as usize].clone(),
            sortition_proof,
        )
    }

    /// Calculate the difficulty for the block to be mined
    // TODO: shall we make a dedicated type for difficulty?
    fn get_difficulty(&self, block_hash: &H256) -> H256 {
        // Get the header of the block corresponding to block_hash
        match self.blockdb.get(block_hash).unwrap() {
            // extract difficulty
            Some(b) => b.header.difficulty,
            None => *DEFAULT_DIFFICULTY,
        }
    }
}

/// Get the current UNIX timestamp
fn get_time() -> u128 {
    let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    match cur_time {
        Ok(v) => {
            return v.as_millis();
        }
        Err(e) => println!("Error parsing time: {:?}", e),
    }
    // TODO: there should be a better way of handling this, or just unwrap and panic
    0
}

#[cfg(test)]
mod tests {}
