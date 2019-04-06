use crate::crypto::hash::H256;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct NodeData {
    /// The chain of the voter node
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

    pub fn is_on_longest_chain(&self) -> bool {
        return self.status == NodeStatus::OnMainChain;
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
    /// Unvoted proposer blocks
    pub unvoted_proposer_blocks: HashMap<u32, H256>,
    /// Minimum level of unvoted proposer block
    pub min_level_unvoted_proposer_block: u32, //inclusive
    /// Maximum level of unvoted proposer block
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
        if self.best_level == level - 1 && self.best_block == block_parent {
            self.best_level += 1;
            self.best_block = block;
            return NodeUpdateStatus::ExtendedMainChain;
        }
        // Rollback required
        else if self.best_level == level - 1 && self.best_block != block_parent {
            panic!("A new fork has emerged");
        }
        // A side_chain block mined.
        else if self.best_level >= level {
            // Do nothing.
            return NodeUpdateStatus::SideChain;
        }
        // Rollback required
        else if self.best_level < level - 1 {
            panic!("A new super longer fork has emerged");
        } else {
            panic!("This should not happen");
        }
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
        } else if level < self.max_level_unvoted_proposer_block + 1 {
            // Ignore. Another proposer mined at 'level'
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

    /// Returns a ordered list of proposer blocks to vote.
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
