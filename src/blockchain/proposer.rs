use crate::crypto::hash::H256;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct NodeData {
    /// Level of the proposer node
    pub level: u32,
    /// Leadership Status
    pub leadership_status: Status,
    /// Number of votes from main-chain voter blocks
    pub votes: u16,
}

impl Default for NodeData {
    fn default() -> Self {
        let level = 0;
        let leadership_status = Status::PotentialLeader;
        return Self {
            level,
            leadership_status,
            votes: 0,
        };
    }
}

impl std::fmt::Display for NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "level: {}; #votes: {}", self.level, self.votes)?; // Ignoring status for now
        Ok(())
    }
}

impl NodeData {
    pub fn increment_vote(&mut self) {
        self.votes += 1;
    }
    pub fn decrement_vote(&mut self) {
        self.votes -= 1;
    }
    pub fn give_leader_status(&mut self) {
        self.leadership_status = Status::Leader
    }
    pub fn give_not_leader_status(&mut self) {
        self.leadership_status = Status::NotLeaderUnconfirmed
    }
    pub fn give_not_leader_confirmed_status(&mut self) {
        self.leadership_status = Status::NotLeaderAndConfirmed
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash, Debug)]
pub enum Status {
    Leader,
    PotentialLeader,       // Will be later used for fast active confirmation.
    NotLeaderUnconfirmed, // When a leader block at a level is confirmed, rest of the proposer block at that level become NotLeaderUnconfirmed
    NotLeaderAndConfirmed, // When a notleader block is confirmed by a one of the child leader block
}

impl NodeData {
    pub fn genesis(number_of_voter_chains: u16) -> Self {
        let mut genesis = NodeData::default();
        genesis.leadership_status = Status::Leader;
        genesis.votes = number_of_voter_chains;
        return genesis;
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Tree {
    /// Best proposer node on the tree chain -- The node with max level. For mining.
    pub best_block: H256,
    /// Best level. For mining.
    pub best_level: u32,
    /// Proposer nodes stored level wise
    pub prop_nodes: Vec<Vec<H256>>,
    /// Votes at each level
    pub number_of_votes: HashMap<u32, u32>,
    /// Stores Leader nodes
    pub leader_nodes: HashMap<u32, H256>, // Using hashmap because leader nodes might not be confirmed sequentially
    /// The level upto which all levels have a leader block.
    pub min_unconfirmed_level: u32,
    /// Pool of unreferred proposer blocks. For mining
    pub unreferred: HashSet<H256>,
}

impl Default for Tree {
    fn default() -> Self {
        let best_block = H256::default();
        let prop_nodes: Vec<Vec<H256>> = vec![];
        let all_votes: HashMap<u32, u32> = HashMap::<u32, u32>::new();
        let leader_nodes: HashMap<u32, H256> = HashMap::new();
        let unreferred: HashSet<H256> = HashSet::new();
        return Self {
            best_block,
            best_level: 0,
            prop_nodes,
            number_of_votes: all_votes,
            leader_nodes,
            min_unconfirmed_level: 1,
            unreferred,
        };
    }
}

impl Tree {
    ///  Adding a proposer block at a level
    pub fn add_block_at_level(&mut self, block: H256, level: u32) {
        if self.best_level >= level {
            self.prop_nodes[level as usize].push(block);
        } else if self.best_level == level - 1 {
            self.prop_nodes.push(vec![block]); // start a new level
            self.best_block = block;
            self.best_level = level;
        } else {
            panic!("Proposer block mined at level without parent block at previous level. Validation fail.")
        }
    }

    /// Adding a vote at a level
    pub fn increment_vote_at_level(&mut self, level: u32) {
        *self.number_of_votes.entry(level).or_insert(0) += 1;
    }

    pub fn insert_unreferred(&mut self, hash: H256) {
        self.unreferred.insert(hash);
    }

    pub fn remove_unreferred(&mut self, hash: &H256) {
        self.unreferred.remove(hash);
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "best_block: {}; best_level: {};",
            self.best_block, self.best_level
        )?; // Ignoring status for now
        Ok(())
    }
}

///// This structure is responsible for confirming blocks. Each level will have its own LeaderConfirmer
///// object which *monitors* the voter chains to see if they have enough votes to confirm a leader
///// block at the level
//pub struct LeaderConfirmer{
//    /// The level of the confirmer
//    pub level: u32,
//    /// Number of votes cast on the level
//    pub number_of_votes: u32,
//    /// The depths of the votes
//    pub depths: Vec<u32>,
//    /// The proposer blocks at the level with lcb of votes.
//    pub proposer_blocks: HashMap<H256, f32>,
//    /// The leader block at the level
//    pub leader_block: Option<H256>
//}
//
//
//
//
//impl LeaderConfirmer{
//    // Initializes a new object.
//    fn new(level: u32, no_of_voting_chains: u16) -> Self {
//        // Initializing zero-vector of no_of_voting_chains size
//        let depths = std::iter::repeat(0).take(no_of_voting_chains as usize).collect::<Vec<u32>>();
//        let proposer_blocks: HashMap<H256, Vec<H256>> = HashMap::<H256, Vec<H256>>::new();
//        let leader_block: Option<H256> = None;
//        return LeaderConfirmer {level, depths, proposer_blocks, leader_block};
//    }
//
//    // Add a new block at level
//    pub fn add_block(&mut self, hash: H256){
//        self.proposer_blocks.insert(hash, 0.0);
//    }
//
//    pub fn add_vote(&mut self, chain_number: u16){
//        self.depths[chain_number as usize] += 1;
//    }
//}
