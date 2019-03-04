//use std::collections::{HashSet};
use super::voter_chain::VoterNode;
use super::utils::*;
use serde::{Serialize, Deserialize};

//#[derive(Clone)]
#[derive(Serialize, Clone, PartialEq)]
pub struct PropNode<'a>{
    /// Block Id
    node_id : BlockId,
    /// Parent prop node
    parent_prop_node: Option<&'a PropNode<'a>>,
    /// Level of the proposer node
    level: u32,
    /// List of Prop nodes which refer this node
    children_prop_node_id: Vec<&'a PropNode<'a>>,
    /// List of Prop nodes referred by this node
    referred_prop_node_ids: Vec<&'a PropNode<'a>>,
    /// List of Voter nodes voted
    votes_node: Vec<&'a VoterNode<'a>>,
    /// Leadership Status
    leadership_status: PropBlockLeaderStatus
}

impl<'a> PropNode<'a>{

    pub fn set_parent(&mut self, parent_prop_node: &'a PropNode<'a>){
        self.parent_prop_node = Some(parent_prop_node);
    }

    pub fn set_level(&mut self, level: u32){
        self.level = level;
    }

    /// Add a prop node which refers 'self'.
    pub fn add_child(&mut self, child_prop_node: &'a PropNode<'a>){
        self.children_prop_node_id.push(child_prop_node);
    }

    /// Add a prop node which is referred by 'self'.
    pub fn add_referred_nodes(&mut self, referred_prop_node: &'a PropNode<'a>){
        self.referred_prop_node_ids.push(referred_prop_node);
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
        let node_id = BlockId::default();
        let parent_prop_node: Option<& PropNode> = None;
        let level = 0;
        let children_prop_node_id: Vec<& PropNode> = vec![];
        let votes_node_ids: Vec<& VoterNode> = vec![];
        let leadership_status = PropBlockLeaderStatus::ConfirmedLeader;
        let referred_prop_node_ids: Vec<& PropNode> = vec![];
        return PropNode {node_id, parent_prop_node, level, children_prop_node_id,
                                    referred_prop_node_ids,
            votes_node: votes_node_ids, leadership_status};
    }

}

impl<'a> Node for PropNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Proposer }
}


/// Stores all the prop nodes
#[derive(Serialize, Clone)]
pub struct PropTree<'a>{
    /// Best node on the main chain
    best_node: Option<&'a PropNode<'a>>,
    /// Proposer nodes stored level wise
    prop_nodes: Vec< Vec<PropNode<'a>> >,
    /// Leader nodes
    leader_nodes : Vec<&'a PropNode<'a>>
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

    /// Get the best node
    pub fn get_best_node(&self) -> Option<&PropNode>{
        return self.best_node;
    }

    /// Get the level of the best node
    pub fn get_best_level(&self) -> u32 {
        return self.best_node.unwrap().level;
    }

    /// Get all the proposer nodes at a level
    pub fn get_all_node_at_level(&self, level: u32) -> Vec<& PropNode> {
        let nodes: &Vec<PropNode> = &self.prop_nodes[level as usize];
        let mut answer: Vec<& PropNode> = vec![];
        for node in nodes{
            answer.push(&node);
        }
        return answer;
    }

    /// Get all potential leader nodes at a level. Used for List Ledger Decoding
    pub fn get_proposer_list_at_level(&self, level: u32) -> Vec<& PropNode> {
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

    /// Add proposer node
    pub fn add_node(&mut self, node: PropNode<'a>) {
        self.prop_nodes[node.level as usize].push(node);
    }

    pub fn set_best_node(&mut self, node: &'a PropNode<'a>){
        self.best_node = Some(node);
    }


}