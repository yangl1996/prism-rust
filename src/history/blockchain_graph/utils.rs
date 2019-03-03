use super::block::block::{BlockType};
use super::crypto::hash::{H256};
use std::collections::{HashSet};

pub enum BlockId {
    Hash(H256),
}

impl std::default::Default for BlockId {
    fn default() -> Self { BlockId::Hash(H256::default()) }
}

pub enum PropBlockLeaderStatus{
    ConfirmedLeader,
    PotentialLeader,
    NotALeader
}

pub enum VoterBlockStatus{
    OnMainChain,
    Orphan
}

// Todo: Import enum block type from block
#[derive(PartialEq)]
pub enum NodeType{
    Transaction,
    Proposer,
    Voter,
}

// Returns the type of the Node
pub trait Node{
    fn get_type() -> NodeType;
}

pub trait Genesis{
    fn get_type() -> Self;
}

/*
Ignore this
/// The status of different blocks.
pub enum TxBlockRefStatus{
    /// When a proposer block has referenced it
    Referenced,
    /// When none of the proposer blocks have referenced it
    UnReferenced
}
pub type PropBlockRefStatus = TxBlockRefStatus;


/// The content of blocks. This is a placeholder until Guilia is done with Mining function.
pub struct TxBlockContent{
    // List of votes on prop blocks.
}

impl Hashable for VoterBlockContent {
    fn sha256(&self) -> SHA256 {
        let x: [u8; 32] = [0; 32]; // Default (wrong) behaviour
        return SHA256(x);
    }
}

pub type PropBlockContent = TxBlockContent;
pub type VoterBlockContent = TxBlockContent;


type TxBlock = Block<TxBlockContent>;
type PropBlock = Block<PropBlockContent>;
type VoterBlock = Block<VoterBlockContent>;

*/