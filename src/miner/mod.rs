pub mod memory_pool;

use crate::block::header::Header;
use crate::block::{proposer, transaction, voter, proof};
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
use crate::validation::get_sortition_id;

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

    for voter_idx in 0..NUM_VOTER_CHAINS {
        let content = voter::Content {
            chain_number: voter_idx as u16,
            voter_parent: VOTER_GENESIS_HASHES[voter_idx as usize],
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
            parent: *PROPOSER_GENESIS_HASH,
            timestamp: get_time(),
            nonce: 0,
            content_merkle_root: H256::default(),
            extra_content: [0; 32],
            difficulty: *DEFAULT_DIFFICULTY,
        },
        contents: contents,
        content_merkle_tree: content_merkle_tree,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    return (ctx, handle);
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
        for voter_chain in 0..NUM_VOTER_CHAINS {
            self.context_update_tx
                .send(ContextUpdateSignal::NewVoterBlock(voter_chain as u16));
        }

        let mut rng = rand::thread_rng();

        // main mining loop
        loop {
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
            // update voter parents
            for voter_chain in new_voter_block.iter() {
                let chain_id: usize = (FIRST_VOTER_INDEX + voter_chain) as usize;
                let voter_parent = self.blockchain.best_voter(*voter_chain as usize);
                if let Content::Voter(c) = &mut self.contents[chain_id] {
                    c.voter_parent = voter_parent;
                    touched_content.insert(chain_id as u16);
                } else {
                    unreachable!();
                }
            }

            // update transaction block content
            if new_transaction_block {
                let mempool = self.mempool.lock().unwrap();
                let transactions = mempool.get_transactions(TX_BLOCK_TRANSACTIONS);
                drop(mempool);
                let chain_id: usize = TRANSACTION_INDEX as usize;
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
                    if c.transaction_refs.len() < PROPOSER_BLOCK_TX_REFS as usize {
                        let mut refs = self.blockchain.unreferred_transactions();
                        refs.truncate(PROPOSER_BLOCK_TX_REFS as usize);
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
                        refs.truncate(PROPOSER_BLOCK_TX_REFS as usize);
                        c.transaction_refs = refs;
                        c.proposer_refs = self.blockchain.unreferred_proposers();
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
                for voter_chain in 0..NUM_VOTER_CHAINS {
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
            } else {
                for voter_chain in new_voter_block.iter() {
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
            if new_proposer_block {
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
                            let empty = {
                                match &mined_block.content {
                                    Content::Transaction(content) => {
                                        if content.transactions.is_empty() {
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                    Content::Voter(content) => {
                                        if content.votes.is_empty() {
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                    Content::Proposer(content) => {
                                        if content.transaction_refs.is_empty()
                                            && content.proposer_refs.is_empty()
                                        {
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                }
                            };
                            empty
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
                    self.server
                        .broadcast(Message::NewBlockHashes(vec![header_hash]));
                    new_validated_block(
                        &mined_block,
                        &self.mempool,
                        &self.blockdb,
                        &self.blockchain,
                        &self.server,
                    );
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
                    thread::sleep(interval);
                }
            }
        }
    }

    /// Given a valid header, sortition its hash and create the block
    fn produce_block(&self, header_hash: H256) -> Block {
        // Get sortition ID
        let sortition_id = get_sortition_id(&header_hash, &self.header.difficulty)
            .expect("Block Hash should <= Difficulty");
        // Create a block
        // get the merkle proof
        let sortition_proof: Vec<H256> = self.content_merkle_tree.proof(sortition_id as usize);
        let mined_block = Block::from_header(
            self.header,
            self.contents[sortition_id as usize].clone(),
            proof::Proof::default(),
        );

        return mined_block;
    }

    /// Calculate the difficulty for the block to be mined
    // TODO: shall we make a dedicated type for difficulty?
    fn get_difficulty(&self, block_hash: &H256) -> H256 {
        // Get the header of the block corresponding to block_hash
        match self.blockdb.get(block_hash).unwrap() {
            // extract difficulty
            Some(b) => {
                return b.header.difficulty;
            }
            None => {
                return *DEFAULT_DIFFICULTY;
            }
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
    return 0;
}

#[cfg(test)]
mod tests {
    use super::memory_pool::MemoryPool;
    use super::{Context, OperatingState};
    use crate::block::tests::{proposer_block, transaction_block, voter_block};
    use crate::block::{proposer, transaction, voter, Content};
    use crate::blockchain::BlockChain;
    use crate::blockdb::BlockDatabase;
    use crate::config;
    use crate::crypto::hash::{Hashable, H256};
    use crate::crypto::merkle::MerkleTree;
    use crate::network::server;
    use crate::transaction::tests as tx_generator;
    use crate::validation::{check_block, BlockResult};
    use std::sync::{Arc, Mutex};

    /*
    #[test]
    fn difficulty() {
        // Initialize a blockchain with 10 voter chains.
        let mempool = Arc::new(Mutex::new(MemoryPool::new()));
        let blockchain = Arc::new(Mutex::new(BlockChain::new(10)));
        let db = Arc::new(BlockDatabase::new(
            &std::path::Path::new("/tmp/prism_miner_check_get_difficulty.rocksdb")).unwrap());
        let (ctx_update_s, ctx_update_r) = channel();
        let (sender, receiver) = channel();
        let (ctx, handle) = new(&mempool, &blockchain, &db, sender, ctx_update_r);
        ctx.start();
        handle.step();
        let block1 = receiver.recv().unwrap();
        handle.exit();
        assert_eq!(block1.header.difficulty, DEFAULT_DIFFICULTY);
    }
    */

    /* this test is commented out for now, since it requires that we add the newly mined blocks to
       the db and the blockchain. if we add those, the test becomes an integration test, and no
       longer fits here.
       TODO: Gerui: but only here can we call a private function
    */

    /*
    // test assemble block and check the block passes validation
    #[test]
    fn assemble_block() {
        let blockdb = BlockDatabase::new("/tmp/prism_test_miner_blockdb.rocksdb").unwrap();
        let blockdb = Arc::new(blockdb);

        let blockchain = BlockChain::new("/tmp/prism_test_miner_blockchain.rocksdb").unwrap();
        let blockchain = Arc::new(blockchain);

        let mempool = Arc::new(Mutex::new(MemoryPool::new(100)));
        let (signal_chan_sender, signal_chan_receiver) = channel();
        let (ctx_update_sink, ctx_update_source) = channel();
        let (msg_tx, msg_rx) = channel();

        let parent = blockchain.best_proposer().unwrap();
        let mut content = vec![];
        content.push(Content::Proposer(proposer::Content::new( vec![], vec![] )));
        content.push(Content::Transaction(transaction::Content::new(vec![])));
        let voter_parent_hash: Vec<H256> = (0..config::NUM_VOTER_CHAINS)
            .map(|i| blockchain.best_voter(i as usize))
            .collect();
        let proposer_block_votes: Vec<Vec<H256>> = (0..config::NUM_VOTER_CHAINS)
            .map(|i| {
                blockchain
                    .unvoted_proposer(&voter_parent_hash[i as usize], &parent)
                    .unwrap()
                    .clone()
            })
            .collect();
        for (i, (voter_parent, proposer_block_votes)) in voter_parent_hash
            .into_iter()
            .zip(proposer_block_votes.into_iter())
            .enumerate()
            {
                content.push(Content::Voter(voter::Content::new(
                    i as u16,
                    voter_parent,
                    proposer_block_votes,
                )));
            }
        let (server_ctx, server) = server::new(
            std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080),
            msg_tx,
        ).unwrap();
        let mut miner = Context {
            mempool: Arc::clone(&mempool),
            blockchain: Arc::clone(&blockchain),
            blockdb: Arc::clone(&blockdb),
            control_chan: signal_chan_receiver,
            context_update_chan: ctx_update_source,
            proposer_parent_hash: parent,
            content_merkle_tree: MerkleTree::new(&content),
            content,
            difficulty: *config::DEFAULT_DIFFICULTY,
            operating_state: OperatingState::Paused,
            server,
        };
        for nonce in 0..100 {
            let mut header = miner.create_header();
            header.nonce = nonce;
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
        for chain in 0..config::NUM_VOTER_CHAINS {
            let voter = voter_block(parent, 3,chain, blockchain.best_voter(chain as usize), vec![]);
            blockdb.insert(&voter);
            blockchain.insert_block(&voter);
            miner.update_voter_content(chain);
            let header = miner.create_header();
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
        miner.update_refed_transaction();
        for nonce in 0..100 {
            let mut header = miner.create_header();
            header.nonce = nonce;
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
        let proposer_1 = proposer_block(parent, 3, vec![], vec![]);
        blockdb.insert(&proposer_1);
        blockchain.insert_block(&proposer_1);
        miner.update_all_contents();
        for nonce in 0..100 {
            let mut header = miner.create_header();
            header.nonce = nonce;
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
    }
    */
}
