/// A proposer tree.

//use std::collections::{HashSet};
use super::voter::VoterNode;
use super::transaction::TxNode;
use super::utils::*;
use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256};

//#[derive(Clone)]
#[derive(Serialize, Clone, PartialEq)]
pub struct PropNode<'a>{
    /// Block Id
    pub node_id : H256,
    /// Parent prop node
    pub parent_prop_node: Option<&'a PropNode<'a>>,
    /// Level of the proposer node
    pub level: u32,
    /// List of Tx nodes this refers
    pub tx_nodes_referred: Vec<&'a TxNode<'a>>,
    /// List of Prop nodes which refer this node
    pub children_prop_nodes: Vec<&'a PropNode<'a>>, // Don't count on this
    /// List of Prop nodes referred by this node
    pub referred_prop_nodes: Vec<&'a PropNode<'a>>,
    /// List of Voter nodes voted on 'Self'
    pub votes_node: Vec<&'a VoterNode<'a>>,
    /// Leadership Status
    pub leadership_status: PropBlockLeaderStatus
}

impl<'a> PropNode<'a>{

    /// Add a prop node which refers 'self'.
    pub fn add_tx_reference(&mut self, tx_node: &'a TxNode<'a>){
        self.tx_nodes_referred.push(tx_node);
    }

    /// Add a prop node which refers 'self'.
    pub fn add_child_node(&mut self, child_prop_node: &'a PropNode<'a>){
        self.children_prop_nodes.push(child_prop_node);
    }

    /// Add a prop node which is referred by 'self'.
    pub fn add_referred_node(&mut self, referred_prop_node: &'a PropNode<'a>){
        self.referred_prop_nodes.push(referred_prop_node);
    }

    /// Add a vote to self
    pub fn add_vote(&mut self, vote_node: &'a VoterNode<'a>){
        self.votes_node.push(vote_node);

    }

    pub fn change_leadership_status(&mut self, new_status: PropBlockLeaderStatus){
        self.leadership_status = new_status;
    }

    pub fn genesis() -> Self{
        return PropNode::default();
    }

    pub fn get_ucb_vote(&self) -> u16 {
        return self.votes_node.len() as u16;
    }

//    pub fsk
}

impl<'a> Default for PropNode<'a> {
    fn default() -> Self {
        let node_id = H256::default();
        let parent_prop_node: Option<& PropNode> = None;
        let level = 0;
        let children_prop_nodes: Vec<& PropNode> = vec![];
        let votes_node: Vec<& VoterNode> = vec![];
        let leadership_status = PropBlockLeaderStatus::ConfirmedLeader;
        let referred_prop_nodes: Vec<&PropNode> = vec![];
        let tx_nodes_referred: Vec<&TxNode> = vec![];
        return PropNode {node_id, parent_prop_node, level,
            tx_nodes_referred, children_prop_nodes,
            referred_prop_nodes, votes_node, leadership_status};
    }

}

impl<'a> Node for PropNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Proposer }
}


/// Stores all the prop nodes
#[derive(Serialize, Clone)]
pub struct PropTree<'a>{
    /// Best node on the main chain
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
        let mut  default_tree: PropTree = PropTree::default();
        // todo: check if the genesis node has level 0.
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

    /// Get all the proposer nodes at a level
    pub fn get_all_node_at_level(&self, level: u32) -> Vec<&PropNode> {
        let nodes: &Vec<PropNode> = &self.prop_nodes[level as usize];
        let mut answer: Vec<& PropNode> = vec![];
        for node in nodes{
            answer.push(&node);
        }
        return answer;
    }

    /// Get all potential leader nodes at a level. Used for List Ledger Decoding
    pub fn get_proposer_list_at_level(&self, level: u32) -> Vec<&PropNode> {
        let all_nodes: Vec<& PropNode> = self.get_all_node_at_level(level);
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
    pub fn get_prop_node_from_node_id(&self, node_id: &H256 ) -> &PropNode {
        unimplemented!();
    }

    /// Add proposer node
    pub fn add_node(&mut self, node: PropNode<'a>) {
        self.prop_nodes[node.level as usize].push(node);
    }
}
