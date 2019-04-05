use crate::crypto::hash::H256;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct NodeData {
    /// Level of the proposer node
    pub level: u32,
    /// Leadership Status
    pub leadership_status: Status,
    /// Number of votes
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
    PotentialLeader,       // Will be used for fast active confirmation
    NotLeaderUnconfirmed, // When a leader block at that level is confirmed, rest become NotLeaderUnconfirmed
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
    pub all_votes: HashMap<u32, u32>, // todo: Inefficient
    /// Stores Leader nodes
    pub leader_nodes: HashMap<u32, H256>, // Using hashmap because leader nodes might not be confirmed sequentially
    /// The level upto which all levels have a leader block.
    pub continuous_leader_level: u32,
    /// The max level at which a leader block exists.
    pub max_leader_level: u32,
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
            all_votes,
            leader_nodes,
            continuous_leader_level: 0,
            max_leader_level: 0,
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
    pub fn add_vote_at_level(&mut self, vote: H256, level: u32) {
        *self.all_votes.entry(level).or_insert(0) += 1;
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
