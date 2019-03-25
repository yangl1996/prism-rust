use crate::transaction::{Transaction};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::blockchain::{BlockChain,NUM_VOTER_CHAINS};
use super::{Block, Content};
use super::header::Header;
use std::collections::HashSet;
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
    blockchain: &'a BlockChain
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
               blockchain: &'a BlockChain ) ->Self {
        Self { proposer_parent_hash, voter_parent_hash, unconfirmed_txs, 
               unreferenced_tx_blocks, unreferenced_prop_blocks, 
               unvoted_proposer_blocks, blockchain }
    }

    // todo: split the function into parts
    pub fn mine(&mut self) -> Block {

        // 1. Creating a merkle tree with m+2 contents ///
        let mut  content = vec![]; // m voter chains, 1 prop and 1 tx blocks
        // Adding m different voter block contents
        for i in 0..NUM_VOTER_CHAINS {
            self.voter_parent_hash[i as usize] = 
                self.blockchain.voter_chains[i as usize].best_block.clone();
            content.push(Content::Voter(voter::Content::new(i,      
                         self.voter_parent_hash[i as usize].clone(),
                         self.unvoted_proposer_blocks[i as usize].clone())));
        }
        // Adding proposer block content
        content.push(Content::Proposer(proposer::Content::new(
                      self.unreferenced_tx_blocks.clone(), self.unreferenced_prop_blocks.clone())));
        // Adding transaction block content
        content.push(Content::Transaction(transaction::Content::new(
                     self.unconfirmed_txs.clone())));
        let content_merkle_tree = MerkleTree::new(&content);

        // 2. Creating a header
        let timestamp: u64 = Miner::get_time();
        let nonce: u32 = 0;

        // Find the correct parents for each type of block
        self.proposer_parent_hash = 
            self.blockchain.proposer_tree.best_block.clone();
        // let difficulty = self.proposer_parent_hash .header.difficulty;
            
        

        let content_root = *content_merkle_tree.root();
        let extra_content :[u8; 32] = [0; 32]; // Add miner id?
        let difficulty :[u8; 32] = [0; 32] ; // todo:This should be proposer_parent's difficulty
        let mut header = Header::new(self.proposer_parent_hash, timestamp , nonce, content_root, extra_content, difficulty);

        // 3. Mining over nonce
        let mut rng = rand::thread_rng();
        let mut sortition_id: u32 ;
        // todo: Use Future feature from rust.
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
        return mined_block;
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

}