pub mod memory_pool;

use crate::block::header::Header;
use crate::block::{proposer, transaction, voter};
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::handler::new_validated_block;
use crate::network::server::Handle as ServerHandle;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::validation::get_sortition_id;
use crate::network::message::Message;

use log::info;

use memory_pool::MemoryPool;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::SystemTime;
use std::time;

use std::sync::{Arc, Mutex};
use std::thread;
use rand::distributions::Distribution;

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
    tx_mempool: Arc<Mutex<MemoryPool>>,
    blockchain: Arc<BlockChain>,
    blockdb: Arc<BlockDatabase>,
    // Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    // Channel for notifying miner of new content
    context_update_chan: Receiver<ContextUpdateSignal>,
    // Proposer parent
    proposer_parent_hash: H256,
    // Block contents
    content: Vec<Content>,
    content_merkle_tree: MerkleTree,
    difficulty: H256,
    operating_state: OperatingState,
    server: ServerHandle,
    proposer_content_transaction_refs: MerkleTree,
    proposer_content_proposer_refs: MerkleTree,
}

#[derive(Clone)]
pub struct Handle {
    // Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    tx_mempool: &Arc<Mutex<MemoryPool>>,
    blockchain: &Arc<BlockChain>,
    blockdb: &Arc<BlockDatabase>,
    ctx_update_source: Receiver<ContextUpdateSignal>,
    server: &ServerHandle,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = channel();
    let ctx = Context {
        tx_mempool: Arc::clone(tx_mempool),
        blockchain: Arc::clone(blockchain),
        blockdb: Arc::clone(blockdb),
        control_chan: signal_chan_receiver,
        context_update_chan: ctx_update_source,
        proposer_parent_hash: H256::default(),
        content: vec![],
        content_merkle_tree: MerkleTree::new(vec![]),
        difficulty: *DEFAULT_DIFFICULTY,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        proposer_content_transaction_refs: MerkleTree::new(vec![]),
        proposer_content_proposer_refs: MerkleTree::new(vec![]),
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
        self.control_chan.send(ControlSignal::Start(lambda, lazy)).unwrap();
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
                info!("Miner starting in continuous mode with lambda {} and lazy mode {}", i, l);
                self.operating_state = OperatingState::Run(i, l);
            }
            ControlSignal::Step => {
                info!("Miner starting in stepping mode");
                self.operating_state = OperatingState::Step;
            }
        }
    }

    fn miner_loop(&mut self) {
        // Initialize the context and the header to mine
        self.update_all_contents();
        let mut header: Header = self.create_header();

        let mut rng = rand::thread_rng();

        // Mining loop
        loop {
            // Check state and incoming control signal
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
            // use try_iter to get multiple messages
            let mut voter_msg = vec![];
            let mut contains_proposer = false;
            let mut contains_transaction = false;
            for sig in self.context_update_chan.try_iter() {
                match sig {
                    ContextUpdateSignal::NewProposerBlock => contains_proposer = true,
                    ContextUpdateSignal::NewVoterBlock(chain) => voter_msg.push(chain),
                    ContextUpdateSignal::NewTransactionBlock => contains_transaction = true,
                };
            }
            if contains_proposer {
                // we hear a proposer block, so update all contents
                self.update_all_contents();
            } else {
                // we didn't hear a proposer block, so don't need to update all contents
                for chain in &voter_msg {
                    self.update_voter_content(*chain);
                }
                if contains_transaction {
                    self.append_refed_transaction();
                    self.update_transaction_content();
                }
            }
            if contains_proposer || contains_transaction || !voter_msg.is_empty() {
                header = self.create_header();
            }

            // try a new nonce, and update the timestamp
            header.nonce = rng.gen(); // random nonce TODO: rng can be slow
            header.timestamp = get_time();

            // Check if we successfully mined a block
            if header.hash() < self.difficulty {
                // Create a block
                let mined_block: Block = self.assemble_block(header);
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
                                        if content.transaction_refs.is_empty() && content.proposer_refs.is_empty() {
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
                    let hash = mined_block.hash();
                    self.blockdb.insert(&mined_block).unwrap();
                    self.server.broadcast(Message::NewBlockHashes(vec![hash]));
                    new_validated_block(
                        &mined_block,
                        &self.tx_mempool,
                        &self.blockdb,
                        &self.blockchain,
                        &self.server,
                    );
                    //                debug!("Mined block {:.8}", mined_block.hash());
                    // if we are stepping, pause the miner loop
                    if let OperatingState::Step = self.operating_state {
                        self.operating_state = OperatingState::Paused;
                    }
                }
                // after we mined this block, we update the context based on this block
                match &mined_block.content {
                    Content::Proposer(_) => self.update_all_contents(),
                    Content::Voter(content) => self.update_voter_content(content.chain_number),
                    Content::Transaction(_) => {
                        self.append_refed_transaction();
                        self.update_transaction_content();
                    }
                }
                header = self.create_header();
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
    fn assemble_block(&self, header: Header) -> Block {
        // Get sortition ID
        let sortition_id = get_sortition_id(&header.hash(), &header.difficulty).expect("Block Hash should <= Difficulty");
        // Create a block
        // get the merkle proof
        let sortition_proof: Vec<H256> = self.content_merkle_tree.proof(sortition_id as usize);
        let mined_block = Block::from_header(
            header,
            self.content[sortition_id as usize].clone(),
            sortition_proof,
        );

        return mined_block;
    }

    /// Create a header object from the current miner view
    fn create_header(&self) -> Header {
        let nonce: u32 = 0; // we will update this value in-place when mining
        let timestamp: u128= get_time();
        let content_merkle_root = self.content_merkle_tree.root();
        let extra_content: [u8; 32] = [0; 32]; // TODO: Add miner id?
        return Header::new(
            self.proposer_parent_hash,
            timestamp,
            nonce,
            content_merkle_root,
            extra_content,
            self.difficulty,
        );
    }

    /// Update the block to be mined
    fn update_all_contents(&mut self) {
        // get mutex of blockchain and get all required data
        let transaction_block_refs = self.blockchain.unreferred_transactions();
        let mut proposer_block_refs = self.blockchain.unreferred_proposers();
        let voter_parent_hash: Vec<H256> = (0..NUM_VOTER_CHAINS)
            .map(|i| self.blockchain.best_voter(i as usize))
            .collect();
        // get best proposer after get unreferred_proposers and voter_parent_hash to avoid they are newer
        // than best proposer
        self.proposer_parent_hash = self.blockchain.best_proposer().unwrap();
        self.difficulty = self.get_difficulty(&self.proposer_parent_hash);
        proposer_block_refs
            .iter()
            .position(|item| *item == self.proposer_parent_hash)
            .map(|i| proposer_block_refs.remove(i));
        let proposer_block_votes: Vec<Vec<H256>> = (0..NUM_VOTER_CHAINS)
            .map(|i| {
                self.blockchain
                    .unvoted_proposer(&voter_parent_hash[i as usize], &self.proposer_parent_hash)
                    .unwrap()
                    .unwrap()
            })
            .collect();
        // get mutex of mempool and get all required data
        let mempool = self.tx_mempool.lock().unwrap();
        let transactions = mempool.get_transactions(TRANSACTION_BLOCK_TX_LIMIT);
        drop(mempool);

        // update the contents and the parents based on current view
        let mut content = vec![];

        // Update proposer content, and create the Merkle trees of two kinds of refs.
        self.proposer_content_transaction_refs = MerkleTree::new(transaction_block_refs.clone());
        self.proposer_content_proposer_refs = MerkleTree::new(proposer_block_refs.clone());
        let proposer_content = proposer::Content::new(
            transaction_block_refs,
            proposer_block_refs,
        );
        let proposer_content_hash = proposer_content.ref_roots_to_hash(self.proposer_content_transaction_refs.root(), self.proposer_content_proposer_refs.root());
        content.push(Content::Proposer(proposer_content));

        // Update transaction content with TX_BLOCK_SIZE mempool txs
        content.push(Content::Transaction(transaction::Content::new(
            transactions,
        )));

        // Update voter content/parents
        for (i, (voter_parent_hash, proposer_block_votes)) in voter_parent_hash
            .into_iter()
            .zip(proposer_block_votes.into_iter())
            .enumerate()
        {
            content.push(Content::Voter(voter::Content::new(
                i as u16,
                voter_parent_hash,
                proposer_block_votes,
            )));
        }

        // we avoid compute twice proposer content hash, there is skip(1)
        let mut hashes = vec![proposer_content_hash];
        hashes.append(&mut content.iter().skip(1).map(|x|x.hash()).collect());
        self.content_merkle_tree = MerkleTree::new(hashes);
        self.content = content;
    }

    /// Update the transaction ref of proposer content, only append the new refs
    fn append_refed_transaction(&mut self) {
        let idx: usize = PROPOSER_INDEX as usize;
        if let Content::Proposer(ref mut content) = self.content.get_mut(idx).unwrap() {
            let mut transaction_block_refs = self.blockchain.unreferred_transactions_diff();
            if !transaction_block_refs.is_empty() {
                // add to content
                content.transaction_refs.extend(&transaction_block_refs);
                // add to the merkle tree, to avoid re-creating merkle tree every time.
                self.proposer_content_transaction_refs.append(&mut transaction_block_refs);
                let content_hash = content.ref_roots_to_hash(self.proposer_content_transaction_refs.root(), self.proposer_content_proposer_refs.root());
                self.content_merkle_tree.update(idx, content_hash);
            }
        } else { unreachable!(); }
    }

    /// Update one voter chain's content with chain number
    fn update_voter_content(&mut self, chain: u16) {
        let idx: usize = (FIRST_VOTER_INDEX + chain) as usize;
        if let Content::Voter(content) = self.content.get(idx).unwrap() {
            let voter_parent = self.blockchain.best_voter(chain as usize);
            if voter_parent != content.voter_parent {
                // we have to check if below function `unvoted_proposer(voter_parent, self.proposer_parent_hash)` result is None or not.
                if let Some(votes) = self.blockchain.unvoted_proposer(&voter_parent, &self.proposer_parent_hash).unwrap() {
                    self.content[idx] = Content::Voter(voter::Content::new(
                            chain,
                            voter_parent,
                            votes,
                            ));
                    self.content_merkle_tree.update(idx, self.content[idx].hash());
                } else {
                    // TODO: this branch means `self.proposer_parent_hash` needs to be updated.
                }
            }
        } else {
            unreachable!();
        }
    }

    /// Update transaction block's content
    fn update_transaction_content(&mut self) {
        let mempool = self.tx_mempool.lock().unwrap();
        let transactions = mempool.get_transactions(TRANSACTION_BLOCK_TX_LIMIT);
        drop(mempool);
        let idx: usize = TRANSACTION_INDEX as usize;
        self.content[idx] = Content::Transaction(transaction::Content::new(
            transactions,
        ));
        self.content_merkle_tree.update(idx, self.content[idx].hash());
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
    use crate::blockdb::BlockDatabase;
    use crate::blockchain::BlockChain;
    use crate::block::{Content, proposer, transaction, voter};
    use crate::block::tests::{proposer_block, voter_block, transaction_block};
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc::channel;
    use super::memory_pool::MemoryPool;
    use super::{Context, OperatingState};
    use crate::config;
    use crate::crypto::hash::{H256, Hashable};
    use crate::crypto::merkle::MerkleTree;
    use crate::network::server;
    use crate::validation::{check_block, BlockResult};
    use crate::transaction::tests as tx_generator;

    /*
    #[test]
    fn difficulty() {
        // Initialize a blockchain with 10 voter chains.
        let tx_mempool = Arc::new(Mutex::new(MemoryPool::new()));
        let blockchain = Arc::new(Mutex::new(BlockChain::new(10)));
        let db = Arc::new(BlockDatabase::new(
            &std::path::Path::new("/tmp/prism_miner_check_get_difficulty.rocksdb")).unwrap());
        let (ctx_update_s, ctx_update_r) = channel();
        let (sender, receiver) = channel();
        let (ctx, handle) = new(&tx_mempool, &blockchain, &db, sender, ctx_update_r);
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
                    .unwrap()
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
            tx_mempool: Arc::clone(&mempool),
            blockchain: Arc::clone(&blockchain),
            blockdb: Arc::clone(&blockdb),
            control_chan: signal_chan_receiver,
            context_update_chan: ctx_update_source,
            proposer_parent_hash: parent,
            content_merkle_tree: MerkleTree::new(content.iter().map(|x|x.hash()).collect()),
            content,
            difficulty: *config::DEFAULT_DIFFICULTY,
            operating_state: OperatingState::Paused,
            server,
            proposer_content_transaction_refs: MerkleTree::new(vec![]),
            proposer_content_proposer_refs: MerkleTree::new(vec![]),
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
        miner.append_refed_transaction();
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
}
