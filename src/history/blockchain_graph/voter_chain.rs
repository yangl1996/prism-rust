use std::collections::{HashSet};
use super::utils::*;
use  super::proposer_tree::PropNode;

pub struct VoterNode<'a>{
    /// The chain of the voter node
    chain_id: u16,
    /// Block Id
    node_id : BlockId,
    /// The parent on its chain
    parent: Option<&'a VoterNode<'a>>,
    /// The parent on proposer tree. This will be used for adaptive difficulty.
    proposer_parent: Option<&'a PropNode<'a>>,
    /// Height from the genesis node
    level: u32,
    /// List of votes on proposer nodes.
    votes: Vec<&'a PropNode<'a>>
}

impl<'a> VoterNode<'a>{
    pub fn set_chain_id(&mut self, chain_id: u16) {self.chain_id = chain_id}

    fn genesis(&self, chain_id: u16) -> Self{
        let mut genesis = VoterNode::default();
        genesis.set_chain_id(chain_id);
        return genesis;
    }
}
impl<'a> Default for VoterNode<'a> {
    fn default() -> Self {
        let chain_id :u16 = 0;
        let node_id = BlockId::default();
        let parent: Option<&'a VoterNode<'a>> = None;
        let proposer_parent: Option<&'a PropNode<'a>> = None;
        let level = 0;
        let votes: Vec<&'a PropNode<'a>> = vec![];
        return VoterNode {chain_id, node_id, parent, proposer_parent, level, votes,};
    }
}


impl<'a> Node for VoterNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Voter }
}


/// Stores all the voter nodes
pub struct VoterChain<'a>{
    /// Voter chain id
    id: u16,
    /// Genesis node
    genesis_node: &'a VoterNode<'a>,
    /// Best node on the main chain
    best_node: &'a VoterNode<'a>,
    /// Set of all Voter nodes
    voter_nodes: HashSet<&'a VoterNode<'a>>
}

impl<'a> VoterChain<'a> {
//    /// Initialize the voter tree
//    pub fn new(id: u16, genesis_node: VoterNode) -> Self {
//        let best_node: VoterNode =  genesis_node.clone(); Todo: Define clone
//        let voter_nodes: HashSet<VoterNode> = HashSet::new();
//        voter_nodes.insert(genesis_node.clone()); Todo: Define insert
//        return VoterChain {id, genesis_node, best_node, voter_nodes};
//    }

    /// Get the best node
    pub fn get_best_node(&self) -> &VoterNode {
        return &self.best_node;
    }

    /// Get the level of the best node
    pub fn get_chain_length(&self) -> &u32 {
        return &self.best_node.level;
    }

    /// Add  voter node
    pub fn add_voter_node(&mut self, node: VoterNode){
//        self.tx_nodes.insert(node); Todo: Define a hash insert??

//        if node.parent == self.best_node{
//            self.best_node = node;
//        }
//        else if node.level > self.best_node.level +1 {
//            //  Todo: Reorg!! Return Success status?
//        }


    }
}