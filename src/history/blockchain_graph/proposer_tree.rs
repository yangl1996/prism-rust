use std::collections::{HashSet};
use super::utils::*;
use super::voter_chain::VoterNode;

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
    votes_node_ids: Vec<&'a VoterNode<'a>>,
    /// Leadership Status
    leadership_status: PropBlockLeaderStatus
}

impl<'a> PropNode<'a>{

    fn set_parent(&mut self, parent_prop_node: &'a PropNode<'a>){
        self.parent_prop_node = Some(parent_prop_node);
    }

    fn set_level(&mut self, level: u32){
        self.level = level;
    }

    /// Add a prop node which refers 'self'.
    fn add_child(&mut self, child_prop_node: &'a PropNode<'a>){
        self.children_prop_node_id.push(child_prop_node);
    }

    /// Add a prop node which is referred by 'self'.
    fn add_referred_nodes(&mut self, referred_prop_node: &'a PropNode<'a>){
        self.referred_prop_node_ids.push(referred_prop_node);
    }

    /// Add a vote to self
    fn add_vote(&mut self, vote_node: &'a VoterNode<'a>){
        self.votes_node_ids.push(vote_node);

    }

    fn change_leadership_status(&mut self, new_status: PropBlockLeaderStatus){
        self.leadership_status = new_status;
    }

    fn genesis(&self) -> Self{
        return PropNode::default();
    }
}

impl<'a> Default for PropNode<'a> {
    fn default() -> Self {
        let node_id = BlockId::default();
        let parent_prop_node: Option<&'a PropNode<'a>> = None;
        let level = 0;
        let children_prop_node_id: Vec<&'a PropNode<'a>> = vec![];
        let votes_node_ids: Vec<&'a VoterNode<'a>> = vec![];
        let leadership_status = PropBlockLeaderStatus::ConfirmedLeader;
        let referred_prop_node_ids: Vec<&'a PropNode<'a>> = vec![];
        return PropNode {node_id, parent_prop_node, level, children_prop_node_id,
                                    referred_prop_node_ids, votes_node_ids, leadership_status};
    }

}

impl<'a> Node for PropNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Proposer }
}


/// Stores all the prop nodes
pub struct PropTree<'a>{
    /// Genesis node
    genesis_node: &'a PropNode<'a>,
    /// Best node on the main chain
    best_node: &'a PropNode<'a>,
    /// Proposer nodes stored level wise
    prop_nodes: Vec< Vec<&'a PropNode<'a>> >,
    /// Leader nodes
    leader_nodes : Vec< Option<&'a PropNode<'a>> >
}

impl<'a> PropTree<'a>{
    /// Get the best node
    pub fn get_best_node(&self) -> &'a PropNode<'a>{
        return &self.best_node;
    }

    /// Get the level of the best node
    pub fn get_best_level(&self) -> &u32 {
        return &self.best_node.level;
    }

    /// Get all the proposer nodes at a level
    pub fn get_all_node_at_level(&self, level: u32) -> &Vec<&'a PropNode<'a>> {
        return &self.prop_nodes[level as usize];
    }

    /// Get all potential leader nodes at a level. Used for List Ledger Decoding
    pub fn get_proposer_list_at_level(&self, level: u32) -> Vec<&'a PropNode<'a>> {
        let all_nodes: &Vec<&'a PropNode<'a>> = self.get_all_node_at_level(level);
        let mut potential_leaders: Vec<&'a PropNode> = Vec::new();
        // Todo: filter proposer nodes with maybe leadership status
        return potential_leaders;
    }

    /// Get the proposer node list sequence up to a level. Used for List Ledger Decoding
    pub fn get_proposer_node_sequence(&self, level: u32) -> Vec<Vec<&PropNode>>{
        let best_level = self.get_best_level();
        let mut proposer_list_sequence :Vec<Vec<&PropNode>> = vec![];
        for l in 0..*best_level {
            proposer_list_sequence.push(self.get_proposer_list_at_level(l));
        }
        return proposer_list_sequence;
    }

    /// Get the leader node at a level
    pub fn get_leader_node_at_level(&self, level: u32) -> &Option<&'a PropNode<'a>>{
        return &self.leader_nodes[level as usize];
    }

    /// Get the leader node sequence up to a level
    pub fn get_leader_node_sequence(&self, level: u32) -> Vec<&Option<&'a PropNode<'a>>>{
        let best_level = self.get_best_level();
        let mut leader_sequence :Vec<&Option<&'a PropNode<'a>>> = vec![];
        for l in 0..*best_level {
            leader_sequence.push(self.get_leader_node_at_level(l));
        }
        return leader_sequence;
    }

    /// Add proposer node
    pub fn add_proposer_node(&mut self, node: &'a PropNode) {
        self.prop_nodes[node.level as usize].push(node);
    }

}