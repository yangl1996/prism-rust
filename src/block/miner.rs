use crate::transaction::{Transaction};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::blockchain::{BlockChain};
use crate::miner::memory_pool::{MemoryPool,Entry};
use super::{Block, Content};
use super::header::Header;
use std::collections::{HashMap};
use super::{transaction, proposer, voter};
use crate::config::*;
use std::time::{SystemTime};
use std::sync::mpsc::{channel,Receiver,Sender,TryRecvError};
use std::thread;

extern crate rand; // 0.6.0
use rand::{Rng};
use rand::prelude::ThreadRng;

extern crate bigint;
use bigint::uint::{U256};

pub struct Miner<'a>{
    // Tx block content
    tx_mempool: &'a MemoryPool,
    // Current blockchain
    blockchain: &'a mut BlockChain,
    // Recent blocks
    seen_blocks: &'a mut HashMap<H256,Block>,
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
               seen_blocks: &'a mut HashMap<H256,Block>,
               outgoing_blocks: Sender<Block>,
               incoming_blocks: Receiver<Block>) -> Self {
        Self { tx_mempool, blockchain, seen_blocks,
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
            if hash < difficulty{
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
        self.seen_blocks.insert(mined_block.hash().clone(), mined_block.clone());

        // update the blockchain
        self.blockchain.insert_node(mined_block);

        // Send the mined block on outgoing blocks channel (update memory)
        // thread::spawn(move || {
        //     self.outgoing_blocks.send(mined_block.clone()).unwrap();
        // });
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
            big_transaction_rate {
            // Transaction block
            return Some(TRANSACTION_INDEX);
        } else if big_hash < big_difficulty / 100.into() *
            (big_transaction_rate + big_proposer_rate) {
            // Proposer block
            return Some(PROPOSER_INDEX);
        } else if big_hash < big_difficulty {
            // Figure out which voter tree we are in
            let voter_id = (big_hash -
                            big_transaction_rate -
                            big_proposer_rate)
                            % NUM_VOTER_CHAINS.into();
            return Some((voter_id).as_u32()+2);
        }
        None
    }

    fn get_difficulty(&self, block_hash: &H256) -> Option<[u8; 32]> {
        // Get the header of the block corresponding to block_hash
        match self.seen_blocks.get(block_hash) {
            // extract difficulty
            Some(block) => return Some(block.header.difficulty.clone()),
            None => return None
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::crypto::hash::{H256};
    use super::*;
    use crate::blockchain::BlockChain;
    use crate::miner::memory_pool::MemoryPool;
    use std::collections::{HashMap};
    use crate::block::{Block};
    use std::sync::mpsc::channel;
    use std::thread;
    use rand::{Rng, RngCore};


    // #[test]
    // fn check_create_header() {

    //     create_header(proposer_parent_hash: &H256,
    //                  content_merkle_tree: &MerkleTree<Content>,
    //                  nonce: &u32,
    //                  difficulty: &[u8;32]) -> Header

    //     let proposer_parent_hash: H256 = 10.into();
    //     let nonce: u32 = 25;
    //     let difficulty: [u8; 32] = 200.into();
    //     let content_merkle_tree: MerkleTree<Content>;

    // }

    #[test]
    fn check_mine() {
        /// Initialize a blockchain with 10  voter chains.
        pub const NUM_VOTER_CHAINS: u16 = 10;
        let mut blockchain = BlockChain::new(NUM_VOTER_CHAINS);
        /// Store the parent blocks to mine on voter trees.
        let mut voter_best_blocks: Vec<H256> =
            (0..NUM_VOTER_CHAINS).map( |i| blockchain.voter_chains[i as usize].best_block).collect(); // Currently the voter genesis blocks.

        println!("Step 1:   Initialized blockchain");
        assert_eq!(11, blockchain.graph.node_count(), "Expecting 11 nodes corresponding to 11 genesis blocks");

        let mut tx_mempool = MemoryPool::new();
        let mut seen_blocks: HashMap<H256,Block> = HashMap::new();
        let (sender, receiver) = channel();
        {
            let mut miner = Miner::new(&mut tx_mempool, &mut blockchain,
                                    &mut seen_blocks,
                                    sender, receiver);
            println!("Step 2:   Initialized miner");
            let block1 = miner.mine();
            let block2 = miner.mine();

            println!("Step 3:   Mined 2 blocks");
            // Check that two blocks are different
            assert!(block1.hash() != block2.hash());
        }

        // Assert that the new block appears in the db
        // TODO: make this check the db for block
        assert_eq!(seen_blocks.len(),2);

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
        assert_eq!(sortition_id,TRANSACTION_INDEX);

        // Set the hash to just below the boundary between tx and proposer blocks
        big_hash = big_difficulty / 100.into() * big_transaction_rate - 1.into();
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,TRANSACTION_INDEX);

        // Proposer blocks
        // Set the hash to just above the boundary between tx and proposer blocks
        big_hash = big_difficulty / 100.into() * big_transaction_rate;
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,PROPOSER_INDEX);

        // Set the hash to just below the boundary between tx and voter blocks
        big_hash = big_difficulty / 100.into() *
            (big_transaction_rate + big_proposer_rate) - 1.into();
        hash = big_hash.into();
        sortition_id = match Miner::get_sortition_id(&hash, &difficulty) {
            Some(u) => u,
            None => 100,
        };
        assert_eq!(sortition_id,PROPOSER_INDEX);

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
