/// A proposer tree.

//use std::collections::{HashSet};
use super::voter::VoterNode;
use super::transaction::TxNode;
use super::status::*;
use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256};

//#[derive(Clone)]
#[derive(Serialize, Clone, PartialEq)]
pub struct PropNode<'a>{
    /// Block Id
    pub block_hash : H256,
    /// Parent prop node
    pub parent_prop_node: Option<&'a PropNode<'a>>,
    /// Level of the proposer node
    pub level: u32,
    /// List of Tx nodes which refers this node
    pub children_tx_nodes: Vec<&'a TxNode<'a>>,
    /// List of Tx nodes referred by this node
    pub referred_tx_nodes: Vec<&'a TxNode<'a>>,
    /// List of Prop nodes which refers this node
    pub children_prop_nodes: Vec<&'a PropNode<'a>>, // Don't count on this
    /// List of Prop nodes referred by this node
    pub referred_prop_nodes: Vec<&'a PropNode<'a>>,
    /// List of Voter nodes voted on 'Self'
    pub votes_node: Vec<&'a VoterNode<'a>>,
    /// Leadership Status
    pub leadership_status: PropBlockLeaderStatus
}

impl<'a> PropNode<'a>{


    /// Add a tx node which refers 'self'.
    pub fn add_child_tx_node(&mut self, tx_node: &'a TxNode<'a>){
        self.children_tx_nodes.push(tx_node);
    }

    /// Add a tx node which is referred by 'self'.
    pub fn add_tx_reference(&mut self, tx_node: &'a TxNode<'a>){
        self.referred_tx_nodes.push(tx_node);
    }

    /// Add a prop node which refers 'self'.
    pub fn add_child_prop_node(&mut self, child_prop_node: &'a PropNode<'a>){
        self.children_prop_nodes.push(child_prop_node);
    }

    /// Add a prop node which is referred by 'self'.
    pub fn add_prop_reference(&mut self, referred_prop_node: &'a PropNode<'a>){
        self.referred_prop_nodes.push(referred_prop_node);
    }

    /// Add a vote node which votes self.
    pub fn add_vote(&mut self, vote_node: &'a VoterNode<'a>){
        self.votes_node.push(vote_node);
    }

    pub fn change_leadership_status(&mut self, new_status: PropBlockLeaderStatus){
        self.leadership_status = new_status;
    }

    pub fn genesis() -> Self{
        return PropNode::default();
    }

    /// Returns the total number of votes
    pub fn get_total_vote(&self) -> u16 {
        return self.votes_node.len() as u16;
    }

    /// Returns effective number of permanent votes with 1 - epsilon guarantee
    pub fn get_lcb_vote(&self, epsilon: f32) -> u16 { unimplemented!(); }
}

impl<'a> Default for PropNode<'a> {
    fn default() -> Self {
        let block_hash = H256::default();
        let parent_prop_node: Option<& PropNode> = None;
        let level = 0;
        let children_prop_nodes: Vec<& PropNode> = vec![];
        let children_tx_nodes: Vec<& TxNode> = vec![];
        let votes_node: Vec<& VoterNode> = vec![];
        let leadership_status = PropBlockLeaderStatus::ConfirmedLeader;
        let referred_prop_nodes: Vec<&PropNode> = vec![];
        let referred_tx_nodes: Vec<&TxNode> = vec![];
        return PropNode {block_hash, parent_prop_node, level, children_tx_nodes, referred_tx_nodes,
                    children_prop_nodes, referred_prop_nodes, votes_node, leadership_status};
    }
}


/// Stores all the proposer nodes
#[derive(Serialize, Clone)]
pub struct PropTree<'a>{
    /// Best proposer node on the tree chain -- The node with max level
    pub best_node: Option<&'a PropNode<'a>>,
    /// Proposer nodes stored level wise
    pub prop_nodes: Vec< Vec<PropNode<'a>> >,
    /// Leader nodes
    pub leader_nodes : Vec<&'a PropNode<'a>>
}

impl<'a>  Default for PropTree<'a> {
    fn default() -> Self {
        let prop_nodes: Vec< Vec<PropNode> > = vec![];
        let leader_nodes: Vec< & PropNode >  = vec![];
        return PropTree {best_node: None, prop_nodes, leader_nodes};
    }
}

impl<'a> PropTree<'a>{
    pub fn new(genesis_node: PropNode<'a>) -> Self {
        let mut default_tree: PropTree = PropTree::default();
        default_tree.add_node(genesis_node);
        return default_tree;
    }

    pub fn genesis_node(&self) -> &PropNode {
        let genesis_node: &PropNode = &self.prop_nodes[0][0];
        return genesis_node;
    }


    /// Get the level of the best node
    pub fn get_best_level(&self) -> u32 {
        return self.best_node.unwrap().level;
    }

    /// Get all the references to proposer nodes at a given level
    pub fn get_all_node_at_level(&self, level: u32) -> Vec<&PropNode> {
        let nodes: &Vec<PropNode> = &self.prop_nodes[level as usize];
        let mut answer: Vec<&PropNode> = vec![];
        for node in nodes{
            answer.push(&node);
        }
        return answer;
    }

    /// Get all potential leader nodes at a level. Used for List Ledger Decoding
    pub fn get_proposer_list_at_level(&self, level: u32) -> Vec<&PropNode> {
        let all_nodes: Vec<& PropNode> = self.get_all_node_at_level(level);
//        let potential_leaders = all_nodes.filter(|&x| x.leadership_status == PropBlockLeaderStatus::PotentialLeader);
        let mut potential_leaders: Vec<& PropNode> = Vec::new();
        for node in all_nodes{
            if node.leadership_status  == PropBlockLeaderStatus::PotentialLeader{
                potential_leaders.push(node);
            }
        }
        // Todo: filter proposer nodes with maybe leadership status
        return potential_leaders;
    }

    /// Get the proposer node list sequence up to a level. Used for List Ledger Decoding
    pub fn get_proposer_node_sequence(&self) -> Vec<Vec<&PropNode>>{
        let best_level = self.get_best_level();
        let mut proposer_list_sequence :Vec<Vec<&PropNode>> = vec![];
        for l in 0..best_level {
            proposer_list_sequence.push(self.get_proposer_list_at_level(l));
        }
        return proposer_list_sequence;
    }

    /// Get the leader node at a level
    pub fn get_leader_node_at_level(&self, level: u32) -> & PropNode{
        return &self.leader_nodes[level as usize];
    }

    /// Get the leader node sequence up to a level
    pub fn get_leader_node_sequence(&self, level: u32) -> Vec<&PropNode>{
        let mut leader_sequence :Vec<& PropNode> = vec![];
        for l in 0..level {
            leader_sequence.push(self.get_leader_node_at_level(l));
        }
        return leader_sequence;
    }

    /// Returns the prop node for the give node id
    /// todo: To yet implement
    pub fn get_prop_node_from_block_hash(&self, block_hash: &H256 ) -> &PropNode {
        unimplemented!();
    }

    /// Add proposer node
    pub fn add_node(&mut self, node: PropNode<'a>) {
        let node_level = node.level;
        self.prop_nodes[node_level as usize].push(node);
        if  node_level > self.best_node.unwrap().level{
//            self.best_node = Some(&self.prop_nodes[node_level as usize].last().unwrap()) //todo: Make this work
//            self.best_node = Some(&node); // todo: Make it work
        }
    }
}
