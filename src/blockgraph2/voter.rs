use crate::crypto::hash::{H256};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct Voter {
    /// The chain of the voter node
    pub chain_number: u16,
    /// Height from the genesis node
    pub level: u32,
    /// Status of the voter block
    pub status: VoterBlockStatus
}

impl Voter{
    pub fn genesis(chain_number: u16) -> Self{
        let mut genesis = Voter::default();
        genesis.chain_number = chain_number;
        genesis.status = VoterBlockStatus::OnMainChain;
        return genesis;
    }
}

impl Default for Voter {
    fn default() -> Self {
        let chain_number :u16 = 0;
        let level = 0;
        let status = VoterBlockStatus::Orphan;
        return Voter {chain_number, level, status};
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum VoterBlockStatus{
    OnMainChain,
    Orphan
}