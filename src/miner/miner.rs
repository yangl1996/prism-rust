use crate::transaction::{Transaction};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::blockchain::{BlockChain};
use crate::config::*;
use crate::block::{Block, Content};
use crate::block::header::Header;
use crate::block::{transaction, proposer, voter};
use crate::blockdb::{BlockDatabase};

use super::memory_pool::{MemoryPool, Entry};
use std::time::{SystemTime, Duration};
use std::sync::mpsc::{channel,Receiver,Sender,TryRecvError};
use std::collections::{HashMap};

use std::sync::{Arc, Mutex};
use std::thread;

extern crate rand; // 0.6.0
use rand::{Rng};
use rand::prelude::ThreadRng;

extern crate bigint;
use bigint::uint::{U256};

/// Signal to the miner
enum Instruction {
    NewContent,
    // TODO: To be added later for efficiency:
    // NewTx,
    // NewTxBlockContent,
    // NewPropBlockContent,
    // NewVoterBlockContent(u16)
}

pub struct Context {
    // Tx mempool
    tx_mempool: Arc<Mutex<MemoryPool>>,
    // Current blockchain
    blockchain: Arc<Mutex<BlockChain>>,
    // Block database
    db: Arc<BlockDatabase>,
    // Channel for receiving control signal
    control_chan: Receiver<Instruction>,
    // Channel for returning newly-mined blocks
    new_block_chan: Sender<Block>,
    // Proposer parent
    proposer_parent_hash: H256,
    // Block contents
    content: Vec<Content>,
    // Content merkle Tree root
    content_merkle_tree_root: H256,
    // Difficulty
    difficulty: [u8; 32]
}

pub struct Handle {
    // Channel for sending signal to the miner thread
    control_chan: Sender<Instruction>,
}

pub fn new(tx_mempool: &Arc<Mutex<MemoryPool>>,
           blockchain: &Arc<Mutex<BlockChain>>,
           db: &Arc<BlockDatabase>,
           block_sink: Sender<Block>) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = channel();
    let ctx = Context {
        tx_mempool: Arc::clone(tx_mempool),
        blockchain: Arc::clone(blockchain),
        db: Arc::clone(db),
        control_chan: signal_chan_receiver,
        new_block_chan: block_sink,
        proposer_parent_hash: H256::default(),
        content: vec![], 
        content_merkle_tree_root: H256::default(),
        difficulty: DEFAULT_DIFFICULTY,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    return (ctx, handle);
}

impl Context {
    pub fn start(mut self) {
        thread::spawn(move || {
            self.miner_loop();
        });
    }

    fn miner_loop (&mut self) {
        // Initialize the context and the header to mine
        self.update_context();
        let mut header: Header = self.create_header();

        let mut rng = rand::thread_rng();

        // Mining loop
        loop {
            // update the header to mine
            header.nonce = rng.gen(); // random nonce TODO: rng can be slow
            header.timestamp = get_time();

            // compute the hash
            let hash: [u8; 32] = (&header.hash()).into(); // TODO: bad code

            // Check if we successfully mined a block
            if hash < self.difficulty {
                // Create a block
                let mined_block: Block = self.assemble_block(header);
                // Release block to the network
                self.new_block_chan.send(mined_block).unwrap();
            }

            // Check for incoming singal
            match self.control_chan.try_recv() {
                Ok(instruction) => {
                    match instruction {
                        Instruction::NewContent => {
                            // TODO: Only update block contents if relevant parent
                            self.update_context();
                            header = self.create_header();
                        },
                    }
                },
                Err(TryRecvError::Empty) => {
                    continue;
                },
                Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
            }
        };
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
        let mined_block = Block::from_header(header, self.content[sortition_id as usize].clone(), sortition_proof);

        return mined_block;
    }

    /// Create a header object from the current miner view
    fn create_header(&self) -> Header {
        let nonce: u32 = 0; // we will update this value in-place when mining
        let timestamp: u64 = get_time();
        let content_root = self.content_merkle_tree_root;
        let extra_content: [u8; 32] = [0; 32]; // TODO: Add miner id?
        return Header::new(self.proposer_parent_hash.clone(),
                           timestamp,
                           nonce,
                           content_root,
                           extra_content,
                           self.difficulty.clone());
    }

    /// Update the block to be mined 
    fn update_context(&mut self) {
        let blockchain = self.blockchain.lock().unwrap();
        self.proposer_parent_hash = blockchain.get_proposer_best_block();
        self.difficulty = self.get_difficulty(&self.proposer_parent_hash);

        // update the contents and the parents based on current view
        let mut content = vec![];

        // TODO: since the content field will always contain three elements, could we switch it to
        // a tuple?
        
        // Update proposer content
        content.push(Content::Proposer(proposer::Content::new(
                        blockchain.get_unreferred_prop_blocks().clone(),
                        blockchain.get_unreferred_tx_blocks().clone())
                    ));

        // Update transaction content with TX_BLOCK_SIZE mempool txs
        let mempool = self.tx_mempool.lock().unwrap();
        content.push(Content::Transaction(transaction::Content::new(
                        mempool
                            .get_transactions(TX_BLOCK_SIZE)
                            .into_iter()
                            .map(|s| s.transaction)
                            .collect())
                    ));
        drop(mempool);

        // Update voter content/parents
        for i in 0..NUM_VOTER_CHAINS {
            content.push(Content::Voter(voter::Content::new(
                            i,
                            blockchain.get_voter_best_block(i as u16).clone(),
                            blockchain.get_unvoted_prop_blocks(i as u16).clone())
                        ));
        }
        drop(blockchain);
        self.content = content;
        let merkle_tree = MerkleTree::new(&self.content);
        self.content_merkle_tree_root = merkle_tree.root();
    }

    /// Calculate which chain should we attach the new block to
    fn get_sortition_id(&self, hash: &[u8; 32]) -> u32 {
        let big_hash = U256::from_big_endian(hash);
        let big_difficulty = U256::from_big_endian(&self.difficulty);
        let big_proposer_rate: U256 = PROPOSER_MINING_RATE.into();
        let big_transaction_rate: U256 = TRANSACTION_MINING_RATE.into();

        if big_hash < big_difficulty / 100.into() * big_proposer_rate {
            // transaction block
            return PROPOSER_INDEX;
        } else if big_hash < big_difficulty / 100.into() * (big_transaction_rate + big_proposer_rate) {
            // proposer block
            return TRANSACTION_INDEX;
        } else if big_hash < big_difficulty {
            // voter index, figure out which voter tree we are in
            let voter_id = (big_hash - big_transaction_rate - big_proposer_rate) % NUM_VOTER_CHAINS.into();
            return voter_id.as_u32() + FIRST_VOTER_INDEX;
        } else {
            panic!("Difficulty {}, The function should not be called for such high value of hash {}",
                   big_difficulty,
                   big_hash);
        }
    }

    /// Calculate the difficulty for the block to be mined
    // TODO: shall we make a dedicated type for difficulty?
    fn get_difficulty(&self, block_hash: &H256) -> [u8; 32] {
        // Get the header of the block corresponding to block_hash
        match self.db.get(block_hash).unwrap() {
            // extract difficulty
            Some(b) => {
                return b.header.difficulty.clone();
            },
            None => {
                return DEFAULT_DIFFICULTY.clone();
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
        },
        Err(e) => println!("Error parsing time: {:?}", e),
    }
    // TODO: there should be a better way of handling this, or just unwrap and panic
    return 0;
}

/*
#[cfg(test)]
mod tests {
    use crate::crypto::hash::{H256};
    use super::*;
    use crate::blockchain::BlockChain;
    use crate::miner::memory_pool::MemoryPool;
    use crate::block::{Block};
    use crate::blockdb::{BlockDatabase};
    use std::sync::mpsc::channel;
    use std::thread;
    use rand::{Rng, RngCore};
    use std::sync::{Arc, Mutex};


    #[test]
    fn get_difficulty() {
        // Initialize a blockchain with 10 voter chains.
        let tx_mempool = Arc::new(Mutex::new(MemoryPool::new()));
        let blockchain = Arc::new(Mutex::new(BlockChain::new()));
        let db = Arc::new(BlockDatabase::new(
            &std::path::Path::new("/tmp/prism_miner_check_get_difficulty.rocksdb")).unwrap());
        let (sender, receiver) = channel();
        let (ctx, handle) = Miner::new(&tx_mempool, &blockchain, &db, sender);
        let block1 = ctx.mine();

        assert_eq!(miner.get_difficulty(&block1.hash()).unwrap(), DEFAULT_DIFFICULTY);

    }

    #[test]
    fn check_mine() {
        /// Initialize a blockchain with 10  voter chains.
        let mut blockchain = BlockChain::new();
        /// Store the parent blocks to mine on voter trees.
        let mut voter_best_blocks: Vec<H256> =
            (0..NUM_VOTER_CHAINS).map( |i| blockchain.voter_chains[i as usize].best_block).collect(); // Currently the voter genesis blocks.

        println!("Step 1:   Initialized blockchain");
        assert_eq!(11, blockchain.graph.node_count(), "Expecting 11 nodes corresponding to 11 genesis blocks");

        let mut tx_mempool = MemoryPool::new();
        let mut db = BlockDatabase::new(
            &std::path::Path::new("/tmp/prismdb.rocksdb")).unwrap();
        let (sender, receiver) = channel();
        {
            let mut miner = Miner::new(&mut tx_mempool, &mut blockchain,  &mut db,  sender, receiver);
            println!("Step 2:   Initialized miner");
            let block1 = miner.mine();
            let block2 = miner.mine();

            println!("Step 3:   Mined 2 blocks");
            // Check that two blocks are different
            assert!(block1.hash() != block2.hash());

            // If the last block was a proposer or voter block, check
            // that it's the best proposer block
            let t = block2.get_block_type().unwrap();
            let block2_hash = block2.hash();
            if t == PROPOSER_INDEX {
                assert_eq!(block2_hash,blockchain.get_proposer_best_block(),
                           "Expected the last-mined proposer block to be the best proposer block");
            } else if t == TRANSACTION_INDEX {
                assert!(blockchain.tx_pool.is_unconfirmed(&block2_hash),
                           "Expected the last-mined tx block to be unconfirmed");
            } else  if t >= FIRST_VOTER_INDEX {
                assert_eq!(blockchain.voter_chains[(t-FIRST_VOTER_INDEX) as usize].best_block, block2_hash,
                           "Expected the last-mined voter block to be the best voter block");
            }
        }

        // Assert that the new blocks appear in the db
        assert_eq!(db.num_blocks(),2,
            "Expecting 2 nodes since 2 blocks were added");

        // Check that 2 new blocks appear in the blockchain
        assert_eq!(blockchain.graph.node_count(), 13,
            "Expecting 13 nodes since 2 blocks were added");

    }

    #[test]
    fn check_get_sortition_id() {
        let difficulty: [u8; 32] = DEFAULT_DIFFICULTY;
        let big_difficulty = U256::from_big_endian(&difficulty);

        let mut big_hash: U256;
        let big_proposer_rate: U256 = PROPOSER_MINING_RATE.into();
        let big_transaction_rate: U256 = TRANSACTION_MINING_RATE.into();

        let mut hash: [u8; 32];
        let mut sortition_id: u32;

        // Transaction blocks
        hash = [0; 32]; // hash = 0
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,PROPOSER_INDEX);

        // Set the hash to just below the boundary between tx and proposer blocks
        big_hash = big_difficulty / 100.into() * big_proposer_rate - 1.into();
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,PROPOSER_INDEX);

        // Proposer blocks
        // Set the hash to just above the boundary between tx and proposer blocks
        big_hash = big_difficulty / 100.into() * big_proposer_rate;
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,TRANSACTION_INDEX);

        // Set the hash to just below the boundary between tx and voter blocks
        big_hash = big_difficulty / 100.into() *
            (big_transaction_rate + big_proposer_rate) - 1.into();
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,TRANSACTION_INDEX);

        // Voter blocks
        // Set the hash to just above the boundary between tx and voter blocks
        big_hash = big_difficulty / 100.into() *
            (big_transaction_rate + big_proposer_rate);
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,FIRST_VOTER_INDEX);

        // Adding NUM_VOTER_CHAINS to the previous hash should
        // give the same result
        big_hash = big_hash + NUM_VOTER_CHAINS.into();
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,FIRST_VOTER_INDEX);

        // Adding one to the previous hash should
        // increment the voter chain  ID by one
        big_hash = big_hash + 1.into();
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,FIRST_VOTER_INDEX+1);

    }

}
*/
