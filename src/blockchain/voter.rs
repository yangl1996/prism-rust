use crate::crypto::hash::H256;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct NodeData {
    /// The chain id of the voter node
    pub chain_number: u16,
    /// Height from the genesis node
    pub level: u32,
    /// Status of the voter block
    pub status: NodeStatus,
}

impl NodeData {
    pub fn genesis(chain_number: u16) -> Self {
        let mut genesis = NodeData::default();
        genesis.chain_number = chain_number;
        genesis.status = NodeStatus::OnMainChain;
        return genesis;
    }

    pub fn is_on_main_chain(&self) -> bool {
        return self.status == NodeStatus::OnMainChain;
    }

    pub fn make_orphan(&mut self) {
        self.status = NodeStatus::Orphan;
    }
    pub fn make_on_main_chain(&mut self) {
        self.status = NodeStatus::OnMainChain;
    }
}

impl Default for NodeData {
    fn default() -> Self {
        let chain_number: u16 = 0;
        let level = 0;
        let status = NodeStatus::Orphan;
        return Self {
            chain_number,
            level,
            status,
        };
    }
}

impl std::fmt::Display for NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CN: {}; level: {}", self.chain_number, self.level)?; // Ignoring status for now
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum NodeStatus {
    OnMainChain,
    Orphan,
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum NodeUpdateStatus {
    ExtendedMainChain,
    SideChain,
    LongerFork,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Chain {
    /// The chain number
    pub chain_number: u16, //todo: remove this?
    /// Best block on the chain
    pub best_block: H256,
    /// Best level
    pub best_level: u32,
    /// Unvoted proposer blocks. For mining.
    pub unvoted_proposer_blocks: HashMap<u32, H256>,
    /// Minimum level of unvoted proposer block. For mining.
    pub min_level_unvoted_proposer_block: u32, //inclusive
    /// Maximum level of unvoted proposer block. For mining.
    pub max_level_unvoted_proposer_block: u32, //inclusive
}

impl Chain {
    pub fn new(chain_number: u16, best_block: H256) -> Self {
        let unvoted_proposer_blocks: HashMap<u32, H256> = HashMap::new();
        return Self {
            chain_number,
            best_block,
            best_level: 0,
            unvoted_proposer_blocks,
            min_level_unvoted_proposer_block: 1,
            max_level_unvoted_proposer_block: 0, //todo: 1,0!!
        };
    }

    pub fn add_voter_block(
        &mut self,
        block: H256,
        block_parent: H256,
        level: u32,
    ) -> NodeUpdateStatus {
        // New best block mined over the previous best block
        if self.best_block == block_parent {
            self.best_level += 1;
            self.best_block = block;
            return NodeUpdateStatus::ExtendedMainChain;
        }
        // A fork of equal length found
        else if self.best_block != block_parent && self.best_level == level - 1{
            println!("found a new fork of equal length");
            return NodeUpdateStatus::SideChain;
        }
        // A side_chain block mined.
        else if self.best_level >= level {
            // Do nothing.
            return NodeUpdateStatus::SideChain;
        }
        // Rollback required
        else if self.best_level < level - 1 {
            return NodeUpdateStatus::LongerFork;
        } else {
            panic!("This should not happen");
        }
    }

    pub fn switch_the_main_chain(&mut self, best_block: H256, best_level: u32){
        self.best_block = best_block;
        self.best_level = best_level;
    }

    /// Adds a proposer to vote at level iff no proposer vote is present at that level.
    pub fn insert_unvoted(&mut self, level: u32, hash: H256) {
        if level == self.max_level_unvoted_proposer_block + 1 {
            if self.unvoted_proposer_blocks.contains_key(&level) {
                panic!("This should have happened");
            }
            self.unvoted_proposer_blocks.insert(level, hash);
            self.max_level_unvoted_proposer_block += 1;
        } else if level > self.max_level_unvoted_proposer_block + 1 {
            panic!(
                "Vote at level {} is skipped",
                self.max_level_unvoted_proposer_block
            );
        } else { // Ignore. Another proposer mined at 'level'
        }
    }

    pub fn remove_unvoted(&mut self, level: u32) {
        if level != self.min_level_unvoted_proposer_block {
            panic!("Votes are not removed in a ordered fashion. Level = {}, min_level={}, max_level={}",
                   level, self.min_level_unvoted_proposer_block, self.max_level_unvoted_proposer_block, );
        }
        self.unvoted_proposer_blocks.remove(&level);
        self.min_level_unvoted_proposer_block += 1;
    }

    /*
     Initial State:
            PB proposer_blocks[0].1
            |
            PB
            |
            PB
            |
            PB proposer_blocks[-1].1
            |
            PB (min_level_unvoted_proposer_block)
            |
            PB
            |
            PB
            |
            PB (max_level_unvoted_proposer_block)

     Final state:
            PB proposer_blocks[0].1 (min_level_unvoted_proposer_block)
            |
            PB
            |
            PB
            |
            PB proposer_blocks[-1].1
            |
            PB
            |
            PB
            |
            PB
            |
            PB (max_level_unvoted_proposer_block)

    */
    pub fn add_unvoted_while_switching(&mut self, left_segment_proposer_votes: Vec<(H256, u32)>) {
        let mut pre_level = left_segment_proposer_votes[0].1;

        self.unvoted_proposer_blocks.insert(pre_level, left_segment_proposer_votes[0].0);
        for proposer_block in left_segment_proposer_votes[1..].iter(){
            let level = proposer_block.1;
            if level!= pre_level+1 {
                panic!("The votes on proposer levels were not continuous");
            }
            self.unvoted_proposer_blocks.insert(level, proposer_block.0);
            pre_level = level;
        }

        //Changing the min_level_unvoted_proposer_block
        if pre_level!= self.min_level_unvoted_proposer_block - 1{
            panic!("The votes are not removed till the min_level_unvoted_proposer_block")
        }
        self.min_level_unvoted_proposer_block = left_segment_proposer_votes[0].1;

    }

    /*
    It removes the proposer blocks voted in the right segment
    */
    pub fn remove_unvoted_while_switching(&mut self, right_segment_proposer_votes: Vec<(H256, u32)>) {
        for vote in right_segment_proposer_votes.iter() {
            self.remove_unvoted(vote.1);
        }
    }

    /// Returns a ordered list of proposer blocks to vote. For mining.
    pub fn get_unvoted_prop_blocks(&self) -> Vec<H256> {
        return (self.min_level_unvoted_proposer_block..=self.max_level_unvoted_proposer_block)
            .map(|level| self.unvoted_proposer_blocks[&level].clone())
            .collect();
    }


}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CN: {}; best_block: {}; best_level: {}",
            self.chain_number, self.best_block, self.best_level
        )?; // Ignoring status for now
        Ok(())
    }
}


/// This structure stores a fork in the voter chain and it is used when a longer fork appears in a voting chain.
/*
            B
            |
            B---- (common parent)
            |   |
            B   B (The segments start on this level)
            |   |
            B   B
            |   |
(left leaf) B   B
                |
                B
                |
                B (right leaf)
*/
pub struct Fork {
    /// Common parent of the two forks
    pub common_parent: H256,
    /// The segment which is currently on the main chain
    pub left_segment: Vec<H256>,
    /// The new segment on the side chain (which is longer than left segment)
    pub right_segment: Vec<H256>
}