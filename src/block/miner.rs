use crate::transaction::{Transaction};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::blockchain::{BlockChain,NUM_VOTER_CHAINS};
use crate::miner::memory_pool::{MemoryPool,Entry};
use super::{Block, Content};
use super::header::Header;
use std::collections::{HashSet,HashMap};
use super::{transaction, proposer, voter};
use crate::config::*;
use std::time::{SystemTime};
use std::sync::mpsc::{channel,Receiver,Sender,TryRecvError};
use std::thread;

extern crate rand; // 0.6.0
use rand::{Rng};

extern crate bigint;
use bigint::uint::{U256};

pub struct Miner<'a>{
    // Proposer block to mine on proposer tree
    proposer_parent_hash: H256,
    // Voter blocks to mine on m different voter trees
    voter_parent_hash: [H256; NUM_VOTER_CHAINS as usize],
    // Ideally Miner `actor' should have access to these three global data.
    // Tx block content
    tx_mempool: &'a MemoryPool,
    // Proposer block contents
    unreferenced_tx_blocks: Vec<H256>, // todo: Should be replaced with tx_block-mem-pool
    unreferenced_prop_blocks: Vec<H256>, // todo: Should be replaced with unreferenced prop_block-mem-pool
    // Voter block content. Each voter chain has its own list of un-voted proper blocks.
    unvoted_proposer_blocks: Vec<Vec<H256>>, // todo: Should be replaced with un_voted_block pool
    // Current blockchain
    blockchain: &'a mut BlockChain,
    // Recent blocks
    seen_blocks: &'a mut HashMap<H256,Block>,
    // Channel for receiving newly-received blocks
    incoming_blocks: Receiver<Block>,
    // Channel for pushing newly-created blocks
    outgoing_blocks: Sender<Block>
}
// todo: Implement default trait

impl<'a> Miner<'a>{
    // This function will be used when the miner is restarted
    pub fn new(proposer_parent_hash: H256,
               voter_parent_hash: [H256; NUM_VOTER_CHAINS as usize],
               tx_mempool: &'a MemoryPool,
               unreferenced_tx_blocks: Vec<H256>,
               unreferenced_prop_blocks: Vec<H256>,
               unvoted_proposer_blocks: Vec<Vec<H256>>,
               blockchain: &'a mut BlockChain,
               seen_blocks: &'a mut HashMap<H256,Block>,
               incoming_blocks: Receiver<Block>,
               outgoing_blocks: Sender<Block>) -> Self {
        Self { proposer_parent_hash, voter_parent_hash, tx_mempool,
               unreferenced_tx_blocks, unreferenced_prop_blocks,
               unvoted_proposer_blocks, blockchain, seen_blocks,
               incoming_blocks, outgoing_blocks }
    }

    pub fn mine(&mut self) {

        // Set the content
        let content = self.update_block_contents();
        let content_merkle_tree = MerkleTree::new(&content);
        let difficulty = self.get_difficulty(self.proposer_parent_hash);


        // Create header
        let mut nonce: u32 = 0;
        let mut rng = rand::thread_rng();
        let mut sortition_id: u32;
        // todo: Use Future feature from rust.
        let mut header = self.create_header(&content_merkle_tree,
            &nonce, &difficulty);

        // Mining loop
        loop{
            let hash: [u8; 32] = (&header.hash()).into(); //todo: bad code
            if hash < difficulty{
                sortition_id = Miner::get_sortition_id(hash, difficulty)
                    .unwrap();
                break;
            }
            // Check if we need to update our block by reading the channel
            match self.incoming_blocks.try_recv() {
                Ok(block) => {
                    // update contents and headers if needed
                    // TODO: Only update block contents if relevant parent/
                    // content actually changed
                    self.update_block_contents();
                },
                Err(TryRecvError::Empty) => {
                    continue;
                },
                Err(TryRecvError::Disconnected) => unreachable!(),
            }
            header.nonce = rng.gen(); // Choosing a random nonce
            header = self.create_header(&content_merkle_tree,
                                                &nonce, &difficulty);
        };

        // Creating a block
        let sortition_proof: Vec<H256> = content_merkle_tree
            .get_proof_from_index(sortition_id)
            .iter()
            .map(|&x| *x)
            .collect();
        let mined_block = Block::from_header(header,
            content[sortition_id as usize].clone(),
            sortition_proof);

        // Add block to the database
        self.release_block(mined_block);
    }

    fn release_block(&mut self, mined_block: Block) {
        // update the block database
        self.seen_blocks.insert(mined_block.hash(), mined_block.clone());

        // update the blockchain
        self.blockchain.insert_node(&mined_block);

        // Send the mined block on outgoing blocks
        // thread::spawn(move || {
        //     self.outgoing_blocks.send(mined_block.clone()).unwrap();
        // });
    }

    fn create_header(&self,
                     content_merkle_tree: &MerkleTree<Content>,
                     nonce: &u32,
                     difficulty: &[u8;32]) -> Header {
        let timestamp: u64 = Miner::get_time();
        let content_root = *content_merkle_tree.root();
        let extra_content :[u8; 32] = [0; 32]; // Add miner id?
        return Header::new(self.proposer_parent_hash,
                           timestamp, nonce.clone(),
                           content_root, extra_content,
                           difficulty.clone());
    }

    fn update_block_contents(&mut self) -> Vec<Content> {
        // update the contents and the parents based on current view
        let mut content = vec![];

        // Update proposer content/parents
        self.proposer_parent_hash =
            self.blockchain.proposer_tree.best_block.clone();
        content.push(Content::Proposer(proposer::Content::new(
                     self.unreferenced_tx_blocks.clone(), self.unreferenced_prop_blocks.clone())));

        // Update transaction content with TX_BLOCK_SIZE mempool txs
        content.push(Content::Transaction(transaction::Content::new(
                     self.tx_mempool
                        .get_transactions(TX_BLOCK_SIZE)
                        .into_iter()
                        .map(|s| s.transaction)
                        .collect())));

        // Update voter content/parents
        for i in 0..NUM_VOTER_CHAINS {
            self.voter_parent_hash[i as usize] =
                self.blockchain.voter_chains[i as usize].best_block.clone();
            content.push(Content::Voter(voter::Content::new(i,
                         self.voter_parent_hash[i as usize].clone(),
                         self.unvoted_proposer_blocks[i as usize].clone())));
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

    fn get_sortition_id(hash: [u8; 32], difficulty: [u8; 32]) -> Option<u32> {
        let bigint_hash = U256::from_big_endian(&hash);
        let bigint_difficulty = U256::from_big_endian(&difficulty);
        let increment: U256 = bigint_difficulty / (NUM_VOTER_CHAINS + 2).into();

        if bigint_hash < increment {
            // Transaction block
            return Some(TRANSACTION_INDEX);
        } else if bigint_hash < increment * 2.into() {
            // Proposer block
            return Some(PROPOSER_INDEX);
        } else if bigint_hash < bigint_difficulty {
            // Figure out which voter tree we are in
            let voter_id = (bigint_hash - increment * 2.into()) / increment;
            // TODO: This will panic if difficulty > 2^32!
            return Some(voter_id.as_u32());
        }
        None
    }

    fn get_difficulty(&self, block_hash: H256) -> [u8; 32] {
        // Get the header of the block corresponding to block_hash
        match self.seen_blocks.get(&block_hash) {
            // extract difficulty
            Some(block) => return block.header.difficulty.clone(),
            None => return [0; 32]
        }
    }
}