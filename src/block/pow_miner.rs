use crate::transaction::{Transaction};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use super::{Block, Content};
use super::header::Header;
use std::collections::HashSet;
use super::{transaction, proposer, voter};

extern crate rand; // 0.6.0
use rand::{Rng};

pub struct Miner{
    /// Proposer block to mine on proposer tree
    proposer_parent_hash: H256,
    /// Voter blocks to mine on m different voter trees
    voter_parent_hash: Vec<H256>,
    /// Ideally Miner `actor' should have access to these three global data.
    /// Tx block content
    unconfirmed_txs: Vec<Transaction>, // todo: Should be replaced with tx-mem-pool
    /// Proposer block content
    unreferenced_tx_blocks: Vec<H256>, // todo: Should be replaced with tx_block-mem-pool
    unreferenced_prop_blocks: Vec<H256>, // todo: Should be replaced with unreferenced prop_block-mem-pool
    /// Voter block content. Each voter chain has a list of un voted proper blocks.
    unvoted_proposer_blocks: Vec<Vec<H256>> // todo: Should be replaced with un_voted_block pool
}

impl Miner{
    // This function will be used when the miner is initialized or restarted
    pub fn new(proposer_parent_hash: H256, voter_parent_hash: Vec<H256>, unconfirmed_txs: Vec<Transaction>,
               unreferenced_tx_blocks: Vec<H256>, unreferenced_prop_blocks: Vec<H256>, unvoted_proposer_blocks: Vec<Vec<H256>> ) ->Self{
        Self {proposer_parent_hash, voter_parent_hash, unconfirmed_txs, unreferenced_tx_blocks, unreferenced_prop_blocks, unvoted_proposer_blocks}
    }


    // todo: split the function into parts
    pub fn mine(&self) -> Block {

        /// 1. Creating a merkle tree of m+2 contents ///
        let m =1000; // todo: Number of chains is fixed for now
        let mut  content = vec![]; // m voter chains, 1 prop and 1 tx blocks
        /// Adding m different voter block contents
        for i in 0..m{
            content.push(Content::Voter(voter::Content::new(i, self.voter_parent_hash[i as usize].clone(),
                                                            self.unvoted_proposer_blocks[i as usize].clone())));
        }
        /// Adding proposer block content
        content.push(Content::Proposer(proposer::Content::new(self.unreferenced_tx_blocks.clone(), self.unreferenced_prop_blocks.clone())));
        /// Adding transaction block content
        content.push(Content::Transaction(transaction::Content::new(self.unconfirmed_txs.clone())));
        let content_merkle_tree = MerkleTree::new(&content);

        /// 2. Creating a header ///
        let timestamp: u64 = 0;
        let nonce: u32 = 0;
        let content_root = *content_merkle_tree.root();
        let extra_content = vec![]; // Add miner id?
        let difficulty :[u8; 32] = [0; 32] ; // todo:This should be proposer_parent's difficulty
        let mut header = Header::new(self.proposer_parent_hash, timestamp , nonce, content_root, extra_content, difficulty);

        /// 3. Mining over nonce
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
            // todo: update the timestamp

        };

        /// 4. Creating a block
        let sortition_proof: Vec<H256> = content_merkle_tree.get_proof_via_index(sortition_id).iter().map(|&x| *x).collect();
        let mined_block = Block::from_header(header, sortition_proof, content[sortition_id as usize].clone());
        return mined_block;
    }

    fn get_sortition_id(hash: [u8; 32]) -> u32 {
        unimplemented!();
    }

}