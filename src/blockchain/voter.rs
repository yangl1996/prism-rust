use crate::crypto::hash::{H256};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct NodeData {
    /// The chain of the voter node
    pub chain_number: u16,
    /// Height from the genesis node
    pub level: u32,
    /// Status of the voter block
    pub status: Status
}

impl NodeData {
    pub fn genesis(chain_number: u16) -> Self{
        let mut genesis = NodeData::default();
        genesis.chain_number = chain_number;
        genesis.status = Status::OnMainChain;
        return genesis;
    }

    pub fn is_on_longest_chain(&self) -> bool{
        return self.status == Status::OnMainChain;
    }
}

impl Default for NodeData {
    fn default() -> Self {
        let chain_number :u16 = 0;
        let level = 0;
        let status = Status::OnMainChain;
        return NodeData {chain_number, level, status};
    }
}


impl std::fmt::Display for NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CN: {}; level: {}", self.chain_number, self.level)?; // Ignoring status for now
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum Status{
    OnMainChain,
    Orphan
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct Chain{
    /// The chain number
    pub chain_number: u16, //todo: remove this?
    /// Best block on the chain
    pub best_block: H256,
    /// Best level
    pub best_level: u32
}

impl Chain{
    pub fn new(chain_number: u16, best_block: H256) -> Self {
        return Self{chain_number, best_block, best_level: 0};
    }

    pub fn update_voter_chain(&mut self, block: H256, block_parent: H256, level: u32) {
//        println!("Level {}. Bets level {}", level, self.best_level);
        // New best block mined over the previous best block
        if self.best_level== level-1 && self.best_block == block_parent  {
//            println!("New best level {} for chain {}",  self.best_level, self.chain_number);
            self.best_level += 1;
            self.best_block =  block;
        }
        // Rollback required
        else if self.best_level== level-1  && self.best_block != block_parent{
            panic!("A new fork has emerged");
        }
        // A side_chain block mined.
        else if self.best_level >= level {
            // Do nothing.
        }
        // Rollback required
        else if self.best_level < level-1 {
            panic!("A new super longer fork has emerged");
        }
        else{
            panic!("This should not happen");
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CN: {}; best_block: {}; best_level: {}",
               self.chain_number, self.best_block, self.best_level)?; // Ignoring status for now
        Ok(())
    }
}

