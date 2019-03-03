use std::collections::{HashSet};
use super::utils::*;
use super::voter_chain::VoterNode;

pub struct PropNode<'a>{
    /// Block Id
    block_id : BlockId,
    /// Parent prop block
    parent_prop_block_id: &'a PropNode<'a>,
    /// Level of the proposer block
    level: u32,
    /// List of Prop blocks which refer this block
    children_prop_block_id: Vec<&'a PropNode<'a>>,
    /// List of Prop blocks referred by this block
    referred_prop_block_ids: Vec<&'a PropNode<'a>>,
    /// List of Voter blocks voted
    votes_block_ids: Vec<&'a VoterNode<'a>>,
    /// Leadership Status
    leadership_status: PropBlockLeaderStatus
}

impl<'a> PropNode<'a>{
    fn change_leadership_status(&mut self, new_status: PropBlockLeaderStatus){
        self.leadership_status = new_status;
    }
}

impl<'a> Node for PropNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Proposer }
}


/// Stores all the prop nodes
pub struct PropTree<'a>{
    /// Genesis block
    genesis_block: &'a PropNode<'a>,
    /// Best block on the main chain
    best_block: &'a PropNode<'a>,
    /// Proposer blocks stored level wise
    prop_nodes: Vec< Vec<&'a PropNode<'a>> >,
    /// Leader blocks
    leader_nodes : Vec< Option<&'a PropNode<'a>> >
}

impl<'a> PropTree<'a>{
    /// Get the best block
    pub fn get_best_block(&self) -> &'a PropNode<'a>{
        return &self.best_block;
    }

    /// Get the level of the best block
    pub fn get_best_level(&self) -> &u32 {
        return &self.best_block.level;
    }

    /// Get all the proposer blocks at a level
    pub fn get_all_block_at_level(&self, level: u32) -> &Vec<&'a PropNode<'a>> {
        return &self.prop_nodes[level as usize];
    }

    /// Get all potential leader blocks at a level. Used for List Ledger Decoding
    pub fn get_proposer_list_at_level(&self, level: u32) -> Vec<&'a PropNode<'a>> {
        let all_blocks: &Vec<&'a PropNode<'a>> = self.get_all_block_at_level(level);
        let mut potential_leaders: Vec<&'a PropNode> = Vec::new();
        // Todo: filter proposer blocks with maybe leadership status
        return potential_leaders;
    }

    /// Get the proposer block list sequence up to a level. Used for List Ledger Decoding
    pub fn get_proposer_block_sequence(&self, level: u32) -> Vec<Vec<&PropNode>>{
        let best_level = self.get_best_level();
        let mut proposer_list_sequence :Vec<Vec<&PropNode>> = vec![];
        for l in 0..*best_level {
            proposer_list_sequence.push(self.get_proposer_list_at_level(l));
        }
        return proposer_list_sequence;
    }

    /// Get the leader block at a level
    pub fn get_leader_block_at_level(&self, level: u32) -> &Option<&'a PropNode<'a>>{
        return &self.leader_nodes[level as usize];
    }

    /// Get the leader block sequence up to a level
    pub fn get_leader_block_sequence(&self, level: u32) -> Vec<&Option<&'a PropNode<'a>>>{
        let best_level = self.get_best_level();
        let mut leader_sequence :Vec<&Option<&'a PropNode<'a>>> = vec![];
        for l in 0..*best_level {
            leader_sequence.push(self.get_leader_block_at_level(l));
        }
        return leader_sequence;
    }

    /// Add proposer block
    pub fn add_proposer_block(&mut self, node: &'a PropNode) {
        self.prop_nodes[node.level as usize].push(node);
    }

}