use crate::crypto::hash::{H256};

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct ProposerNodeData {
    /// Level of the proposer node
    pub level: u32,
    /// Leadership Status
    pub leadership_status: PropBlockLeaderStatus,
    /// Number of votes
    pub votes: u16,
}

impl Default for ProposerNodeData {
    fn default() -> Self {
        let level = 0;
        let leadership_status = PropBlockLeaderStatus::NotALeader;
        return ProposerNodeData {level, leadership_status, votes: 0};
    }
}

impl std::fmt::Display for ProposerNodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "level: {}; #votes: {}", self.level, self.votes)?; // Ignoring status for now
        Ok(())
    }
}

impl ProposerNodeData{
    pub fn increment_vote(&mut self){
        self.votes += 1;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum PropBlockLeaderStatus{
    ConfirmedLeader,
    PotentialLeader,
    NotALeader
}

impl ProposerNodeData {
    pub fn genesis(number_of_voter_chains: u16) -> Self{
        let mut genesis = ProposerNodeData::default();
        genesis.leadership_status = PropBlockLeaderStatus::ConfirmedLeader;
        genesis.votes = number_of_voter_chains;
        return genesis;
    }
    /// Returns effective number of permanent votes with 1 - epsilon guarantee
    pub fn get_lcb_vote(&self, epsilon: f32) -> u16 { unimplemented!(); }
}

#[derive(Serialize, Deserialize, Clone, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct ProposerTree{
    /// Best proposer node on the tree chain -- The node with max level
    pub best_block: H256,
    /// Best level
    pub best_level: u32,
    /// Proposer nodes stored level wise
    pub prop_nodes: Vec< Vec<H256> >,
    /// Votes at each level
    pub all_votes: Vec< Vec<H256> >, // Can be removed
    /// Leader nodes
    pub leader_nodes : Vec<Option<H256>> // functionality not implemented
}

impl Default for ProposerTree {
    fn default() -> Self {
        let best_block = H256::default();
        let prop_nodes :Vec< Vec<H256> > = vec![];
        let all_votes :Vec< Vec<H256> > = vec![];
        let leader_nodes :Vec<Option<H256>> = vec![];
        return ProposerTree {best_block, best_level:0, prop_nodes, all_votes, leader_nodes};
    }
}

impl ProposerTree{
    ///  Adding a proposer block at a level
    pub fn add_block_at_level(&mut self, block: H256, level: u32){
//        println!("prop tree num levels {}", self.prop_nodes.len());
        if self.prop_nodes.len() >= (level + 1) as usize {
            self.prop_nodes[level as usize].push(block);
        } else if self.prop_nodes.len()== level as usize {
            self.prop_nodes.push(vec![block]); // start a new level
            self.best_block = block;
            self.best_level = level;
        } else{
            panic!("Proposer block mined at level without parent block at previous level")
        }
    }

    /// Adding a vote at a level
    pub fn add_vote_at_level(&mut self, vote: H256, level: u32){
//        println!("prop tree votes levels {}. Level of vote added {}", self.all_votes.len(), level);
        if self.all_votes.len() >= (level + 1) as usize {
            self.all_votes[level as usize].push(vote);
        } else if self.all_votes.len() == level as usize {
            self.all_votes.push(vec![vote]);
        } else{
            panic!("Proposer block mined at level without parent block at previous level")
        }
    }
}

impl std::fmt::Display for ProposerTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "best_block: {}; best_level: {};",
               self.best_block, self.best_level)?; // Ignoring status for now
        Ok(())
    }
}