//use super::block::block::{BlockType}; todo: reuse
use crate::crypto::hash::{H256};
use serde::{Serialize, Deserialize};

/// Different statuses of the proposer nodes.
#[derive(Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum PropBlockLeaderStatus{
    ConfirmedLeader,
    PotentialLeader,
    NotALeader
}

/// Different statuses of voter blocks.
#[derive(Copy, Clone)]
pub enum VoterBlockStatus{
    OnMainChain,
    Orphan
}