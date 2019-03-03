use std::collections::{HashSet};
use super::utils::*;



pub struct VoterNode<'a>{
    /// The chain of the voter block
    chain_id: u16,
    /// Block Id
    block_id : BlockId,
    /// The parent on its chain
    parent: &'a VoterNode<'a>,
    /// Height from the genesis block
    level: u32
}


impl<'a> Node for VoterNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Voter }
}


/// Stores all the voter nodes
pub struct VoterChain<'a>{
    /// Voter chain id
    id: u16,
    /// Genesis block
    genesis_block: &'a VoterNode<'a>,
    /// Best block on the main chain
    best_block: &'a VoterNode<'a>,
    /// Set of all Voter nodes
    voter_nodes: HashSet<&'a VoterNode<'a>>
}

impl<'a> VoterChain<'a> {
//    /// Initialize the voter tree
//    pub fn new(id: u16, genesis_block: VoterNode) -> Self {
//        let best_block: VoterNode =  genesis_block.clone(); Todo: Define clone
//        let voter_nodes: HashSet<VoterNode> = HashSet::new();
//        voter_nodes.insert(genesis_block.clone()); Todo: Define insert
//        return VoterChain {id, genesis_block, best_block, voter_nodes};
//    }

    /// Get the best block
    pub fn get_best_block(&self) -> &VoterNode {
        return &self.best_block;
    }

    /// Get the level of the best block
    pub fn get_chain_length(&self) -> &u32 {
        return &self.best_block.level;
    }

    /// Add  voter block
    pub fn add_voter_block(&mut self, node: VoterNode){
//        self.tx_nodes.insert(node); Todo: Define a hash insert??

//        if node.parent == self.best_block{
//            self.best_block = node;
//        }
//        else if node.level > self.best_block.level +1 {
//            //  Todo: Reorg!! Return Success status?
//        }


    }
}