use super::utils::*;
use super::transaction_pool::TxPool;
use super::proposer_tree::PropTree;
use super::voter_chain::VoterChain;
use super::block::block::{Block, BlockType};
use super::crypto::hash::{Hashable};

pub struct BlockChainGraph<'a>{
    tx_block_pool: &'a TxPool<'a>,
    prop_block_tree: &'a PropTree<'a>,
    voter_chains: Vec<&'a VoterChain<'a>>
}

impl<'a> BlockChainGraph<'a>{

//    pub fn new(number_of_voter_chains: u32) -> Self {
//        let mut voter_chains: Vec<VoterChain> =  vec![];
//
//        /// Initializing voter chains
//        for i in 0..number_of_voter_chains{
//            //Todo: Generate random genesis block
//            // let v_chain = VoterChain::new(i, genesis_block);
//            // voter_chains.push(v_chain);
//        }
//
//        let tx_block_pool: TxPool = TxPool::new();
//        //Todo: Generate random genesis block
//        // let prop_block_tree: TxPool = PropTree::new();
//
//        return BlockChainGraph{}
//    }

    pub fn get_number_of_voter_chains(&self) -> u32{
        return self.voter_chains.len() as u32;
    }

    // todo: T must be a trait which accepts only blocks
    pub fn add_block_as_node<T: Hashable>(&mut self, block: Block<T>){
        if block.get_block_type() == BlockType::Transaction{

//            self.tx_block_pool.add_tx_node()
        }
    }
}