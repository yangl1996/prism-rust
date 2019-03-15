/// A voter chain.

//use std::collections::{HashSet}; todo: use this later
use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256};

use super::status::*;
use super::proposer::PropNode;


//#[derive(Clone)]
#[derive(Serialize, Clone)]
pub struct VoterNode<'a>{
    /// The chain of the voter node
    pub chain_number: u16,
    /// Block Id
    pub block_hash : H256,
    /// The (voter )parent on its chain
    pub parent_voter_node: Option<&'a VoterNode<'a>>,
    /// The parent on proposer tree. This will be used to adaptively adjust the difficulty.
    pub parent_prop_node: Option<&'a PropNode<'a>>,
    /// Height from the genesis node
    pub level: u32,
    /// List of votes on proposer nodes.
    pub votes: Vec<&'a PropNode<'a>>
}

impl<'a> VoterNode<'a>{
    pub fn genesis(chain_number: u16) -> Self{
        let mut genesis = VoterNode::default();
        genesis.chain_number = chain_number;
        return genesis;
    }

    pub fn add_vote(&mut self, vote: &'a PropNode<'a>){
        self.votes.push(vote);
    }
}

impl<'a> Default for VoterNode<'a> {
    fn default() -> Self {
        let chain_number :u16 = 0;
        let block_hash = H256::default();
        let chain_parent_node: Option<& VoterNode> = None;
        let parent_prop_node: Option<& PropNode> = None;
        let level = 0;
        let votes: Vec<& PropNode> = vec![];
        return VoterNode {chain_number, block_hash, parent_voter_node: chain_parent_node, parent_prop_node, level, votes};
    }
}

impl<'a> PartialEq for VoterNode<'a> {
    fn eq(&self, other: &VoterNode) -> bool {
        self.block_hash == other.block_hash
    }
}

/// Stores all the voter nodes
#[derive(Serialize, Clone)]
pub struct VoterChain<'a>{
    /// Voter chain id
    pub chain_number: u16,
    /// Best node on the main chain
    pub best_node: Option<&'a VoterNode<'a>>,
    /// Set of all Voter nodes
    pub voter_nodes: Vec<VoterNode<'a>> //todo: Do we want to move the nodes into this ?
}

impl<'a>  Default for VoterChain<'a> {
    fn default() -> Self {
        let voter_nodes: Vec<VoterNode> = vec![];
        return VoterChain {chain_number: 0, best_node: None, voter_nodes};
    }
}

impl<'a> VoterChain<'a>{
    pub fn new(genesis_node: VoterNode<'a>) -> Self {
        let mut default_chain: VoterChain = VoterChain::default();
        default_chain.add_node(genesis_node);
        return default_chain;
    }

    pub fn genesis_node(&self) -> &VoterNode {
        let genesis_node: &VoterNode = &self.voter_nodes[0];
        return genesis_node;
    }

    /// Get the level of the best node
    pub fn get_chain_length(&self) -> u32 {
        return self.best_node.unwrap().level;
    }

    /// Returns the voter node for the give block_hash
    /// todo: To yet implement
    pub fn get_voter_node_from_block_hash(&self, block_hash: &H256 ) -> &VoterNode {
        unimplemented!();
    }

    /// Add  voter node
    pub fn add_node(&mut self, node: VoterNode<'a>){
        self.voter_nodes.push(node); //Todo: Define a hash insert??

//        if node.parent == self.best_node{
//            self.best_node = Some(&node);
//        }
//        else if node_level > self.best_node.unwrap().level +1 {
//            //  Todo: Reorg!! Return Success enum status?
//        }

    }
}
