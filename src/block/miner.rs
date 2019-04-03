use crate::transaction::{Transaction};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::blockchain::{BlockChain};
use crate::miner::memory_pool::{MemoryPool,Entry};
use crate::blockdb::{BlockDatabase};
use crate::config::*;
use super::{Block, Content};
use super::{transaction, proposer, voter};
use super::header::Header;

use std::time::{SystemTime};
use std::sync::mpsc::{channel,Receiver,Sender,TryRecvError};
use std::thread;


extern crate rand; // 0.6.0
use rand::{Rng};
use rand::prelude::ThreadRng;

extern crate bigint;
use bigint::uint::{U256};

pub struct Miner<'a>{
    // Tx mempool
    tx_mempool: &'a MemoryPool,
    // Current blockchain
    blockchain: &'a mut BlockChain,
    // Block database
    db: &'a mut BlockDatabase,
    // Channel for receiving newly-received blocks
    incoming_blocks: Receiver<Block>,
    // Channel for pushing newly-created blocks
    outgoing_blocks: Sender<Block>,
    // Random number generator
    rng: ThreadRng
}

impl<'a> Miner<'a>{
    // This function will be used when the miner is restarted
    pub fn new(tx_mempool: &'a MemoryPool,
               blockchain: &'a mut BlockChain,
               db: &'a mut BlockDatabase,
               outgoing_blocks: Sender<Block>,
               incoming_blocks: Receiver<Block>) -> Self {
        Self { tx_mempool, blockchain, db,
               outgoing_blocks, incoming_blocks,
               rng: rand::thread_rng() }
    }

    pub fn mine(&mut self) -> Block {

        // Initialize the content
        let mut proposer_parent_hash =
            self.blockchain.get_proposer_best_block();
        let content = self.update_block_contents();
        let content_merkle_tree = MerkleTree::new(&content);
        let mut difficulty = match self.get_difficulty(&proposer_parent_hash) {
            Some(d) => d,
            None => DEFAULT_DIFFICULTY,
        };
        // Initialize header variable
        // todo: Use Future feature from rust.
        let mut header: Header;

        // Mining loop
        loop{
            // Create a new header with random nonce
            header = self.create_header(&proposer_parent_hash,
                                        &content_merkle_tree,
                                        &difficulty);
            // Compute the hash
            let hash: [u8; 32] = (&header.hash()).into(); //todo: bad code

            // Check hash difficulty
            if hash < difficulty {
                // Create a block
                let mined_block = self.create_block(&hash, &difficulty,
                                                    content_merkle_tree,
                                                    &content, header);
                // Release block to the network
                self.release_block(&mined_block);
                return mined_block;
            }
            // Check if we need to update our block by reading the channel
            match self.incoming_blocks.try_recv() {
                Ok(block) => {
                    // update contents and headers if needed
                    // TODO: Only update block contents if relevant parent/
                    // content actually changed
                    proposer_parent_hash =
                        self.blockchain.get_proposer_best_block();
                    let content = self.update_block_contents();
                    let content_merkle_tree = MerkleTree::new(&content);
                    difficulty = match self.get_difficulty(&proposer_parent_hash) {
                        Some(d) => d,
                        None => DEFAULT_DIFFICULTY,
                    };
                },
                Err(TryRecvError::Empty) => {
                    continue;
                },
                Err(TryRecvError::Disconnected) => unreachable!(),
            }
        };
    }

    fn create_block(&mut self, hash: &[u8; 32], difficulty: &[u8; 32],
                     content_merkle_tree: MerkleTree<Content>,
                     content: &Vec<Content>, header: Header) -> Block {
        // Get sortition ID
        let sortition_id = Miner::get_sortition_id(hash, difficulty)
            .unwrap();
        // Create a block
        let sortition_proof: Vec<H256> = content_merkle_tree
            .get_proof_from_index(sortition_id);
        let mined_block = Block::from_header(header,
            content[sortition_id as usize].clone(),
            sortition_proof);

        return mined_block;
    }

    fn release_block(&mut self, mined_block: &Block) {

        // update the block database
        self.db.insert(&mined_block.hash(), mined_block);

        // update the blockchain
        self.blockchain.insert_node(mined_block);

        // Send on the outgoing blocks channel
        self.outgoing_blocks.send(mined_block.clone()).unwrap();
    }

    fn create_header(&mut self, proposer_parent_hash: &H256,
                     content_merkle_tree: &MerkleTree<Content>,
                     difficulty: &[u8;32]) -> Header {
        // Choose a random nonce
        let nonce: u32 = self.rng.gen();
        let timestamp: u64 = Miner::get_time();
        let content_root = content_merkle_tree.root();
        let extra_content: [u8; 32] = [0; 32]; // Add miner id?
        return Header::new(proposer_parent_hash.clone(),
                           timestamp, nonce,
                           content_root, extra_content,
                           difficulty.clone());
    }

    fn update_block_contents(&self) -> Vec<Content> {
        // update the contents and the parents based on current view
        let mut content = vec![];

        // Update proposer content
        content.push(Content::Proposer(proposer::Content::new(
                     self.blockchain.get_unreferred_prop_blocks().clone(),
                     self.blockchain.get_unreferred_tx_blocks().clone())));

        // Update transaction content with TX_BLOCK_SIZE mempool txs
        content.push(Content::Transaction(transaction::Content::new(
                     self.tx_mempool
                        .get_transactions(TX_BLOCK_SIZE)
                        .into_iter()
                        .map(|s| s.transaction)
                        .collect(),)));

        // Update voter content/parents
        for i in 0..NUM_VOTER_CHAINS {
            content.push(Content::Voter(voter::Content::new(i,
                         self.blockchain.get_voter_best_block(i as u16)
                            .clone(),
                         self.blockchain.get_unvoted_prop_blocks(i as u16)
                            .clone()))
                        );
        }
        return content;
    }

    fn get_time() -> u64 {
        let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
        match cur_time {
            Ok(v) => {
                return v.as_secs();
            },
            Err(e) => println!("Error parsing time: {:?}", e),
        }
        return 0;
    }

    fn get_sortition_id(hash: &[u8; 32], difficulty: &[u8; 32]
                       ) -> Option<u32> {
        let big_hash = U256::from_big_endian(hash);
        let big_difficulty = U256::from_big_endian(difficulty);
        let big_proposer_rate: U256 = PROPOSER_MINING_RATE.into();
        let big_transaction_rate: U256 = TRANSACTION_MINING_RATE.into();

        if big_hash < big_difficulty / 100.into() *
            big_proposer_rate {
            // Transaction block
            return Some(PROPOSER_INDEX);
        } else if big_hash < big_difficulty / 100.into() *
            (big_transaction_rate + big_proposer_rate) {
            // Proposer block
            return Some(TRANSACTION_INDEX);
        } else if big_hash < big_difficulty {
            // Figure out which voter tree we are in
            let voter_id = (big_hash -
                            big_transaction_rate -
                            big_proposer_rate)
                            % NUM_VOTER_CHAINS.into();
            return Some((voter_id).as_u32()+FIRST_VOTER_INDEX);
        }
        None
    }

    fn get_difficulty(&self, block_hash: &H256) -> Option<[u8; 32]> {
        // Get the header of the block corresponding to block_hash
        match self.db.get(block_hash).unwrap() {
            // extract difficulty
            Some(b) => {
                return Some(b.header.difficulty.clone());
            },
            None => {
                // TODO: Add genesis blocks to db so we don't need to do this
                return Some(DEFAULT_DIFFICULTY.clone())
            }
        }
    }
}


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


    #[test]
    fn check_get_difficulty() {

        /// Initialize a blockchain with 10  voter chains.
        let mut blockchain = BlockChain::new();
        /// Store the parent blocks to mine on voter trees.
        let mut voter_best_blocks: Vec<H256> =
            (0..NUM_VOTER_CHAINS).map( |i| blockchain.voter_chains[i as usize].best_block).collect();

        let mut tx_mempool = MemoryPool::new();
        let mut db = BlockDatabase::new(
            &std::path::Path::new("/tmp/prismdb.rocksdb")).unwrap();
        let (sender, receiver) = channel();
        let mut miner = Miner::new(&mut tx_mempool, &mut blockchain,
                                &mut db,
                                sender, receiver);
        let block1 = miner.mine();

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
            &std::path::Path::new("/tmp/block_miner_tests_check_mine.rocksdb")).unwrap();
        let (sender, receiver) = channel();
        {
            let mut miner = Miner::new(&mut tx_mempool, &mut blockchain,
                                    &mut db,
                                    sender, receiver);
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
                assert_eq!(blockchain.voter_chains[(t-FIRST_VOTER_INDEX) as usize].best_block,                  block2_hash,
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
