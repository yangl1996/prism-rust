use crate::crypto::hash::{H256};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct VoterNodeData {
    /// The chain of the voter node
    pub chain_number: u16,
    /// Height from the genesis node
    pub level: u32,
    /// Status of the voter block
    pub status: VoterBlockStatus
}

impl VoterNodeData {
    pub fn genesis(chain_number: u16) -> Self{
        let mut genesis = VoterNodeData::default();
        genesis.chain_number = chain_number;
        genesis.status = VoterBlockStatus::OnMainChain;
        return genesis;
    }
}

impl Default for VoterNodeData {
    fn default() -> Self {
        let chain_number :u16 = 0;
        let level = 0;
        let status = VoterBlockStatus::Orphan;
        return VoterNodeData {chain_number, level, status};
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum VoterBlockStatus{
    OnMainChain,
    Orphan
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct VoterChain{
    /// The chain number
    pub chain_number: u16, //todo: remove this?
    /// Best block on the chain
    pub best_block: H256,
    /// Best level
    pub best_level: u32
}

impl VoterChain{
    pub fn new(chain_number: u16, best_block: H256) -> Self {
        return Self{chain_number, best_block, best_level: 0};
    }

    pub fn update_voter_chain(&mut self, block: H256, block_parent: H256, level: u32) {
        // New best block mined over the preivous best block
        if self.best_level== level+1 && self.best_block == block_parent  {
            self.best_level += 1;
            self.best_block =  block;
        }
        // Rollback required
        else if self.best_level== level+1  && self.best_block != block_parent{
            panic!("A new longer fork has emerged");
        }
        // A side_chain block mined.
        else if self.best_level < (level +1) {
            // Do nothing.
        }
        // Rollback required
        else if self.best_level > (level +1) {
            panic!("A new super longer fork has emerged");
        }
        else{
            panic!("This should not happen");
        }
    }
}
