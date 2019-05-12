pub mod memory_pool;

use crate::block::header::Header;
use crate::block::{proposer, transaction, voter};
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::utxodb::UtxoDatabase;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::handler::new_validated_block;
use crate::network::server::Handle as ServerHandle;
use crate::wallet::Wallet;
use log::{info, debug};

use memory_pool::MemoryPool;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::SystemTime;

use std::sync::{Arc, Mutex};
use std::thread;

extern crate rand; // 0.6.0
use rand::Rng;

extern crate bigint;
use bigint::uint::U256;

#[derive(PartialEq)]
enum ControlSignal {
    Start,
    Step,
    Exit,
}

#[derive(PartialEq)]
pub enum ContextUpdateSignal {
    NewContent,
    // TODO: To be added later for efficiency:
    // NewTx,
    // NewTxBlockContent,
    // NewPropBlockContent,
    // NewVoterBlockContent(u16)
}

#[derive(PartialEq)]
enum OperatingState {
    Paused,
    Run,
    Step,
    ShutDown,
}

pub struct Context {
    tx_mempool: Arc<Mutex<MemoryPool>>,
    blockchain: Arc<BlockChain>,
    utxodb: Arc<UtxoDatabase>,
    wallet: Arc<Wallet>,
    db: Arc<BlockDatabase>,
    // Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    // Channel for notifying miner of new content
    context_update_chan: Receiver<ContextUpdateSignal>,
    // Proposer parent
    proposer_parent_hash: H256,
    // Block contents
    content: Vec<Content>,
    content_merkle_tree_root: H256,
    difficulty: H256,
    operating_state: OperatingState,
    server: ServerHandle,
}

pub struct Handle {
    // Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    tx_mempool: &Arc<Mutex<MemoryPool>>,
    blockchain: &Arc<BlockChain>,
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
    db: &Arc<BlockDatabase>,
    ctx_update_source: Receiver<ContextUpdateSignal>,
    server: ServerHandle,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = channel();
    let ctx = Context {
        tx_mempool: Arc::clone(tx_mempool),
        blockchain: Arc::clone(blockchain),
        utxodb: Arc::clone(utxodb),
        wallet: Arc::clone(wallet),
        db: Arc::clone(db),
        control_chan: signal_chan_receiver,
        context_update_chan: ctx_update_source,
        proposer_parent_hash: H256::default(),
        content: vec![],
        content_merkle_tree_root: H256::default(),
        difficulty: *DEFAULT_DIFFICULTY,
        operating_state: OperatingState::Paused,
        server: server,
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

    pub fn start(&self) {
        self.control_chan.send(ControlSignal::Start).unwrap();
    }

    pub fn step(&self) {
        self.control_chan.send(ControlSignal::Step).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        info!("Miner initialized and paused");
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            }).unwrap();
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start => {
                info!("Miner starting");
                self.operating_state = OperatingState::Run;
            }
            ControlSignal::Step => {
                info!("Miner starting in stepping mode");
                self.operating_state = OperatingState::Step;
            }
        }
    }

    fn miner_loop(&mut self) {
        // Initialize the context and the header to mine
        self.update_context();
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
            if self.operating_state == OperatingState::ShutDown {
                return;
            }

            // check whether there is new content
            match self.context_update_chan.try_recv() {
                Ok(_) => {
                    // TODO: Only update block contents of the relevant structures
                    self.update_context();
                    header = self.create_header();
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => panic!("Miner context update channel detached"),
            }

            // try a new nonce, and update the timestamp
            header.nonce = rng.gen(); // random nonce TODO: rng can be slow
            header.timestamp = get_time();

            // Check if we successfully mined a block
            if header.hash() < self.difficulty {
                // Create a block
                let mined_block: Block = self.assemble_block(header);
                //if the mined block is an empty tx block, we ignore it, and go straight to next mining loop
                match &mined_block.content {
                    Content::Transaction(content) => {
                        if content.transactions.is_empty() {
                            continue;
                        }
                    }
                    Content::Voter(content) => {
                        if content.votes.is_empty() {
                            continue;
                        }
                    }
                    _ => (),
                }
                // Release block to the network
                new_validated_block(
                    &mined_block,
                    &self.tx_mempool,
                    &self.db,
                    &self.blockchain,
                    &self.server,
                    &self.utxodb,
                    &self.wallet,
                );
                debug!("Mined one block");
                // TODO: Only update block contents if relevant parent
                self.update_context();
                header = self.create_header();
                // if we are stepping, pause the miner loop
                if self.operating_state == OperatingState::Step {
                    self.operating_state = OperatingState::Paused;
                }
            }
        }
    }

    /// Given a valid header, sortition its hash and create the block
    fn assemble_block(&self, header: Header) -> Block {
        // Get sortition ID
        let hash: [u8; 32] = (&header.hash()).into();
        let sortition_id = self.get_sortition_id(&hash);
        // Create a block
        // assemble the merkle tree and get the proof
        let merkle_tree = MerkleTree::new(&self.content);
        let sortition_proof: Vec<H256> = merkle_tree.get_proof_from_index(sortition_id);
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
        let timestamp: u64 = get_time();
        let content_root = self.content_merkle_tree_root;
        let extra_content: [u8; 32] = [0; 32]; // TODO: Add miner id?
        return Header::new(
            self.proposer_parent_hash.clone(),
            timestamp,
            nonce,
            content_root,
            extra_content,
            self.difficulty,
        );
    }

    /// Update the block to be mined
    fn update_context(&mut self) {
        // get mutex of blockchain and get all required data
        self.proposer_parent_hash = self.blockchain.best_proposer();
        self.difficulty = self.get_difficulty(&self.proposer_parent_hash);
        let transaction_block_refs = self.blockchain.unreferred_transaction(); //.clone();
        let proposer_block_refs = self.blockchain.unreferred_proposer().clone(); // remove clone?
        let voter_parent_hash: Vec<H256> = (0..NUM_VOTER_CHAINS)
            .map(|i| self.blockchain.best_voter(i as usize).clone())
            .collect();
        let proposer_block_votes: Vec<Vec<H256>> = (0..NUM_VOTER_CHAINS)
            .map(|i| self.blockchain.unvoted_proposer(&voter_parent_hash[i as usize]).unwrap().clone())
            .collect();
        // get mutex of mempool and get all required data
        let mempool = self.tx_mempool.lock().unwrap();
        let transactions = mempool.get_transactions(TX_BLOCK_SIZE);
        drop(mempool);

        // update the contents and the parents based on current view
        let mut content = vec![];

        // TODO: since the content field will always contain three elements, could we switch it to
        // a tuple?

        // Update proposer content
        content.push(Content::Proposer(proposer::Content::new(
            transaction_block_refs,
            proposer_block_refs,
        )));

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

        self.content = content;
        let merkle_tree = MerkleTree::new(&self.content);
        self.content_merkle_tree_root = merkle_tree.root();
    }

    /// Calculate which chain should we attach the new block to
    fn get_sortition_id(&self, hash: &[u8; 32]) -> u32 {
        let big_hash = U256::from_big_endian(hash);
        let difficulty_raw: [u8; 32] = (&self.difficulty).into();
        let big_difficulty = U256::from_big_endian(&difficulty_raw);
        let big_proposer_rate: U256 = PROPOSER_MINING_RATE.into();
        let big_transaction_rate: U256 = TRANSACTION_MINING_RATE.into();

        if big_hash < big_difficulty / 100.into() * big_proposer_rate {
            // transaction block
            return PROPOSER_INDEX;
        } else if big_hash
            < big_difficulty / 100.into() * (big_transaction_rate + big_proposer_rate)
        {
            // proposer block
            return TRANSACTION_INDEX;
        } else if big_hash < big_difficulty {
            // voter index, figure out which voter tree we are in
            let voter_id =
                (big_hash - big_transaction_rate - big_proposer_rate) % NUM_VOTER_CHAINS.into();
            return voter_id.as_u32() + FIRST_VOTER_INDEX;
        } else {
            panic!(
                "Difficulty {}, The function should not be called for such high value of hash {}",
                big_difficulty, big_hash
            );
        }
    }

    /// Calculate the difficulty for the block to be mined
    // TODO: shall we make a dedicated type for difficulty?
    fn get_difficulty(&self, block_hash: &H256) -> H256 {
        // Get the header of the block corresponding to block_hash
        match self.db.get(block_hash).unwrap() {
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
fn get_time() -> u64 {
    let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    match cur_time {
        Ok(v) => {
            return v.as_secs();
        }
        Err(e) => println!("Error parsing time: {:?}", e),
    }
    // TODO: there should be a better way of handling this, or just unwrap and panic
    return 0;
}

#[cfg(test)]
mod tests {

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

    // this test is commented out for now, since it requires that we add the newly mined blocks to
    // the db and the blockchain. if we add those, the test becomes an integration test, and no
    // longer fits here.
    /*
    #[test]
    fn mine() {
        // Initialize a blockchain with 10 voter chains.
        let tx_mempool = Arc::new(Mutex::new(MemoryPool::new()));
        let blockchain = Arc::new(Mutex::new(BlockChain::new()));
        let db = Arc::new(BlockDatabase::new(
            &std::path::Path::new("/tmp/prism_miner_mine.rocksdb")).unwrap());
        let (sender, receiver) = channel();
        let (ctx, handle) = new(&tx_mempool, &blockchain, &db, sender);
        ctx.start();

        // check whether blockchain is correctly initialized
        assert_eq!(11, blockchain.lock().unwrap().graph.node_count(), "Expecting 11 nodes corresponding to 11 genesis blocks");

        // mine two blocks
        handle.step();
        let block1 = receiver.recv().unwrap();
        handle.step();
        let block2 = receiver.recv().unwrap();
        handle.exit();

        // check those two blocks are different
        assert!(block1.hash() != block2.hash());

        // If the last block was a proposer or voter block, check that it's the best proposer block
        let t = block2.get_block_type().unwrap();
        let block2_hash = block2.hash();
        if t == PROPOSER_INDEX {
            assert_eq!(block2_hash, blockchain.lock().unwrap().get_proposer_best_block(),
                       "Expected the last-mined proposer block to be the best proposer block");
        } else if t == TRANSACTION_INDEX {
            assert!(blockchain.lock().unwrap().tx_pool.is_unconfirmed(&block2_hash),
                       "Expected the last-mined tx block to be unconfirmed");
        } else  if t >= FIRST_VOTER_INDEX {
            assert_eq!(blockchain.lock().unwrap().voter_chains[(t-FIRST_VOTER_INDEX) as usize].best_block, block2_hash,
                       "Expected the last-mined voter block to be the best voter block");
        }

        // Assert that the new blocks appear in the db
        assert_eq!(db.num_blocks(),2,
            "Expecting 2 nodes since 2 blocks were added");

        // Check that 2 new blocks appear in the blockchain
        assert_eq!(blockchain.lock().unwrap().graph.node_count(), 13,
            "Expecting 13 nodes since 2 blocks were added");

    }
    */

    /*
    #[test]
    fn sortition_id() {
        let tx_mempool = Arc::new(Mutex::new(MemoryPool::new()));
        let (state_update_sink, state_update_source) = mpsc::channel();
        let blockchain = Arc::new(Mutex::new(BlockChain::new(
            NUM_VOTER_CHAINS,
            state_update_sink,
        )));
        let db = Arc::new(
            BlockDatabase::new(&std::path::Path::new(
                "/tmp/prism_miner_test_sortition.rocksdb",
            ))
            .unwrap(),
        );
        let (_ctx_update_s, ctx_update_r) = channel();
        let (ctx, _handle) = new(&tx_mempool, &blockchain, &db, ctx_update_r);

        let big_difficulty = U256::from_big_endian(&DEFAULT_DIFFICULTY);

        let mut big_hash: U256;
        let big_proposer_rate: U256 = PROPOSER_MINING_RATE.into();
        let big_transaction_rate: U256 = TRANSACTION_MINING_RATE.into();

        let mut hash: [u8; 32];
        let mut sortition_id: u32;

        // Transaction blocks
        hash = [0; 32]; // hash = 0
        sortition_id = ctx.get_sortition_id(&hash);
        assert_eq!(sortition_id, PROPOSER_INDEX);

        // Set the hash to just below the boundary between tx and proposer blocks
        big_hash = big_difficulty / 100.into() * big_proposer_rate - 1.into();
        hash = big_hash.into();
        sortition_id = ctx.get_sortition_id(&hash);
        assert_eq!(sortition_id, PROPOSER_INDEX);

        // Proposer blocks
        // Set the hash to just above the boundary between tx and proposer blocks
        big_hash = big_difficulty / 100.into() * big_proposer_rate;
        hash = big_hash.into();
        sortition_id = ctx.get_sortition_id(&hash);
        assert_eq!(sortition_id, TRANSACTION_INDEX);

        // Set the hash to just below the boundary between tx and voter blocks
        big_hash =
            big_difficulty / 100.into() * (big_transaction_rate + big_proposer_rate) - 1.into();
        hash = big_hash.into();
        sortition_id = ctx.get_sortition_id(&hash);
        assert_eq!(sortition_id, TRANSACTION_INDEX);

        // Voter blocks
        // Set the hash to just above the boundary between tx and voter blocks
        big_hash = big_difficulty / 100.into() * (big_transaction_rate + big_proposer_rate);
        hash = big_hash.into();
        sortition_id = ctx.get_sortition_id(&hash);
        assert_eq!(sortition_id, FIRST_VOTER_INDEX);

        // Adding NUM_VOTER_CHAINS to the previous hash should
        // give the same result
        big_hash = big_hash + NUM_VOTER_CHAINS.into();
        hash = big_hash.into();
        sortition_id = ctx.get_sortition_id(&hash);
        assert_eq!(sortition_id, FIRST_VOTER_INDEX);

        // Adding one to the previous hash should
        // increment the voter chain  ID by one
        big_hash = big_hash + 1.into();
        hash = big_hash.into();
        sortition_id = ctx.get_sortition_id(&hash);
        assert_eq!(sortition_id, FIRST_VOTER_INDEX + 1);
    }
    */
}
