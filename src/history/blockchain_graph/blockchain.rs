use super::utils::*;
use super::transaction_pool::{TxPool, TxNode};
use super::proposer_tree::{PropTree, PropNode};
use super::voter_chain::{VoterChain, VoterNode};
use super::block::block::{Block, BlockType};
use super::crypto::hash::{Hashable};

pub struct BlockChainGraph<'a>{
    tx_block_pool: TxPool<'a>,
    prop_block_tree: PropTree<'a>,
    voter_chains: Vec<VoterChain<'a>>
}

impl<'a> BlockChainGraph<'a>{

    pub fn new(number_of_voter_chains: u32) -> Self {

        let tx_block_pool: TxPool = TxPool::new();

        let prop_genesis_block: PropNode = PropNode::genesis();
        let prop_block_tree: PropTree = PropTree::new(prop_genesis_block);

        let mut voter_chains: Vec<VoterChain> =  vec![];

        for i in 0..number_of_voter_chains{
            let voter_genesis_block: VoterNode = VoterNode::genesis(i as u16);
            let voter_chain: VoterChain= VoterChain::new(voter_genesis_block);
            voter_chains.push(voter_chain);
        }

        return BlockChainGraph{tx_block_pool, prop_block_tree, voter_chains}
    }

    pub fn get_number_of_voter_chains(&self) -> u32{
        return self.voter_chains.len() as u32;
    }

    // todo: T must be a trait which accepts only blocks
    pub fn add_block_as_node<T: Hashable>(&mut self, block: Block<T>){

        if block.get_block_type() == BlockType::Transaction{
            // todo: Convert the content into node data. Waiting for Guilia's code.
            let tmp_tx_node = TxNode::default();
            self.tx_block_pool.add_node(tmp_tx_node);
        }
        else if block.get_block_type() == BlockType::Voter{
            // todo: Convert the content into node data. Waiting for Guilia's code.
            // todo: Extract the voter chain. Waiting for Guilia's code.
            let tmp_voter_node = VoterNode::default();
            let i = 0;
            self.voter_chains[i as usize].add_node(tmp_voter_node);
        }
        if block.get_block_type() == BlockType::Proposer{
            // todo: Convert the content into node data. Waiting for Guilia's code.
            let tmp_prop_node = PropNode::default();
            self.prop_block_tree.add_node(tmp_prop_node);
        }
    }
}