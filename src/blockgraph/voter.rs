/// A voter chain.

//use std::collections::{HashSet}; todo: use this later
use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256};

use super::utils::*;
use super::proposer::PropNode;


//#[derive(Clone)]
#[derive(Serialize, Clone)]
pub struct VoterNode<'a>{
    /// The chain of the voter node
    pub chain_id: u16,
    /// Block Id
    pub node_id : H256,
    /// The parent on its chain
    pub chain_parent_node: Option<&'a VoterNode<'a>>,
    /// The parent on proposer tree. This will be used for adaptive difficulty.
    pub parent_prop_node: Option<&'a PropNode<'a>>,
    /// Height from the genesis node
    pub level: u32,
    /// List of votes on proposer nodes.
    pub votes: Vec<&'a PropNode<'a>>
//    ///
//    voter_chain:
}

impl<'a> VoterNode<'a>{
    pub fn genesis(chain_id: u16) -> Self{
        let mut genesis = VoterNode::default();
        genesis.chain_id = chain_id;
        return genesis;
    }

    pub fn add_vote(&mut self, vote: &'a PropNode<'a>){
        self.votes.push(vote);
    }
}

impl<'a> Default for VoterNode<'a> {
    fn default() -> Self {
        let chain_id :u16 = 0;
        let node_id = H256::default();
        let chain_parent_node: Option<& VoterNode> = None;
        let parent_prop_node: Option<& PropNode> = None;
        let level = 0;
        let votes: Vec<& PropNode> = vec![];
        return VoterNode {chain_id, node_id, chain_parent_node, parent_prop_node, level, votes};
    }
}

impl<'a> PartialEq for VoterNode<'a> {
    fn eq(&self, other: &VoterNode) -> bool {
        self.node_id == other.node_id
    }
}

impl<'a> Node for VoterNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Voter }
}


/// Stores all the voter nodes
#[derive(Serialize, Clone)]
pub struct VoterChain<'a>{
    /// Voter chain id
    pub chain_id: u16,
    /// Best node on the main chain
    pub best_node: Option<&'a VoterNode<'a>>,
    /// Set of all Voter nodes
    pub voter_nodes: Vec<VoterNode<'a>> //todo: Do we want to move the nodes into this ?
}

impl<'a>  Default for VoterChain<'a> {
    fn default() -> Self {
        let voter_nodes: Vec<VoterNode> = vec![];
        return VoterChain {chain_id: 0, best_node: None, voter_nodes};
    }
}

impl<'a> VoterChain<'a>{
    pub fn new(genesis_node: VoterNode<'a>) -> Self {
        let mut  default_chain: VoterChain = VoterChain::default();
        default_chain.add_node(genesis_node);
        return default_chain;
    }

    pub fn genesis_node(&self) -> &VoterNode {
        let genesis_node: &VoterNode = &self.voter_nodes[0];
        return genesis_node;
    }

    /// Get the best node
    pub fn get_best_node(&self) -> Option<& VoterNode> {
        return self.best_node;
    }

    /// Get the level of the best node
    pub fn get_chain_length(&self) -> u32 {
        return self.best_node.unwrap().level;
    }


    /// Returns the prop node for the give node id
    /// todo: To yet implement
    pub fn get_voter_node_from_node_id(&self, node_id: &H256 ) -> &VoterNode {
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
