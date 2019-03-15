use crate::crypto::hash::{H256};

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct Proposer {
    /// Level of the proposer node
    pub level: u32,
    /// Leadership Status
    pub leadership_status: PropBlockLeaderStatus,
    /// Number of votes
    pub votes: u16,
    /// Number of voter on all block at the level
    pub total_level_votes: u16
}

impl Default for Proposer {
    fn default() -> Self {
        let level = 0;
        let leadership_status = PropBlockLeaderStatus::NotALeader;
        return Proposer {level, leadership_status, votes: 0, total_level_votes: 0};
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum PropBlockLeaderStatus{
    ConfirmedLeader,
    PotentialLeader,
    NotALeader
}

impl Proposer{
    pub fn genesis(number_of_voter_chains: u16) -> Self{
        let mut genesis = Proposer::default();
        genesis.leadership_status = PropBlockLeaderStatus::ConfirmedLeader;
        genesis.votes = number_of_voter_chains;
        genesis.total_level_votes = number_of_voter_chains;
        return genesis;
    }
}