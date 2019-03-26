use crate::transaction::{Transaction};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::blockchain::{BlockChain,NUM_VOTER_CHAINS};
use super::{Block, Content};
use super::header::Header;
use std::collections::{HashSet,HashMap};
use super::{transaction, proposer, voter};
use std::time::{SystemTime};

extern crate rand; // 0.6.0
use rand::{Rng};

pub struct Miner<'a>{
    // Proposer block to mine on proposer tree
    proposer_parent_hash: H256,
    // Voter blocks to mine on m different voter trees
    voter_parent_hash: [H256; NUM_VOTER_CHAINS as usize],
    // Ideally Miner `actor' should have access to these three global data.
    // Tx block content
    unconfirmed_txs: Vec<Transaction>, // todo: Should be replaced with tx-mem-pool
    // Proposer block contents
    unreferenced_tx_blocks: Vec<H256>, // todo: Should be replaced with tx_block-mem-pool
    unreferenced_prop_blocks: Vec<H256>, // todo: Should be replaced with unreferenced prop_block-mem-pool
    // Voter block content. Each voter chain has its own list of un-voted proper blocks.
    unvoted_proposer_blocks: Vec<Vec<H256>>, // todo: Should be replaced with un_voted_block pool
    // Current blockchain
    blockchain: &'a BlockChain,
    // Recent blocks
    seen_blocks: &'a mut HashMap<H256,Block>
}
// todo: Implement default trait

impl<'a> Miner<'a>{
    // This function will be used when the miner is restarted
    pub fn new(proposer_parent_hash: H256,
               voter_parent_hash: [H256; NUM_VOTER_CHAINS as usize],
               unconfirmed_txs: Vec<Transaction>,
               unreferenced_tx_blocks: Vec<H256>,
               unreferenced_prop_blocks: Vec<H256>,
               unvoted_proposer_blocks: Vec<Vec<H256>>,
               blockchain: &'a BlockChain,
               seen_blocks: &'a mut HashMap<H256,Block>) -> Self {
        Self { proposer_parent_hash, voter_parent_hash, unconfirmed_txs,
               unreferenced_tx_blocks, unreferenced_prop_blocks,
               unvoted_proposer_blocks, blockchain, seen_blocks }
    }

    // todo: split the function into parts
    pub fn mine(&mut self) -> Block {

        // Set the content
        let content = self.update_block_contents();
        let content_merkle_tree = MerkleTree::new(&content);
        let difficulty = self.get_difficulty(self.proposer_parent_hash);


        // Create header
        let mut nonce: u32 = 0;
        let mut rng = rand::thread_rng();
        let mut sortition_id: u32 ;
        // todo: Use Future feature from rust.
        let mut header = self.create_header(&content_merkle_tree,
            &nonce, &difficulty);
        loop{
            let hash: [u8; 32] = (&header.hash()).into(); //todo: bad code
            if hash < difficulty{
                sortition_id = Miner::get_sortition_id(hash);
                break;
            }
            header.nonce = rng.gen(); // Choosing a random nonce

        };

        // 4. Creating a block
        let sortition_proof: Vec<H256> = content_merkle_tree.get_proof_from_index(sortition_id).iter().map(|&x| *x).collect();
        let mined_block = Block::from_header(header, content[sortition_id as usize].clone(),  sortition_proof);

        // 5. Add block to the database
        self.seen_blocks.insert(mined_block.hash(), mined_block.clone());

        return mined_block;
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

        // Update transaction content
        content.push(Content::Transaction(transaction::Content::new(
                     self.unconfirmed_txs.clone())));

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

    fn get_sortition_id(hash: [u8; 32]) -> u32 {
        unimplemented!();
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