use crate::crypto::hash::H256;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
/// The metadata of a voter block.
pub struct NodeData {
    /// The chain ID of the block.
    pub chain_number: u16,
    /// The level of the block.
    pub level: u32,
    /// Status of the block.
    pub status: NodeStatus,
}

impl NodeData {
    /// Generate the metadata for the genesis voter block at the given chain number.
    pub fn genesis(chain_number: u16) -> Self {
        let mut genesis = NodeData::default();
        genesis.chain_number = chain_number;
        genesis.status = NodeStatus::OnMainChain;
        return genesis;
    }

    /// Check whether the block is on the main chain (longest chain).
    pub fn is_on_main_chain(&self) -> bool {
        return self.status == NodeStatus::OnMainChain;
    }

    /// Mark the block as orphan.
    pub fn make_orphan(&mut self) {
        self.status = NodeStatus::Orphan;
    }

    /// Mark the block as on the main chain.
    pub fn make_on_main_chain(&mut self) {
        self.status = NodeStatus::OnMainChain;
    }
}

// TODO: get rid of it?
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
/// The status of a voter block.
pub enum NodeStatus {
    /// The block is on the main chain (longest chain).
    OnMainChain,
    /// The block is an orphan block.
    Orphan,
}

#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
/// The result of inserting a new voter block to the chain.
pub enum NodeUpdateStatus {
    /// The new block extends the main chain.
    ExtendedMainChain,
    /// The new block attaches to a side chain.
    SideChain,
    /// The new block extends a side chain and it now becomes the main chain.
    LongerFork,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
/// The metadata of a voter chain.
pub struct Chain {
    /// The hash of the deepest block on this chain.
    pub best_block: H256,
    /// The level of the deepest block.
    pub best_level: u32,
    /// The hash of the proposer block that we should vote for but have not been voted at each
    /// level.
    pub unvoted_proposer_blocks: HashMap<u32, H256>,
    /// The smallest level on the proposer chain that has not been voted. This is for mining.
    pub min_level_unvoted_proposer_block: u32,
    /// The greatest level on the proposer chain that has not been voted. This is for mining.
    pub max_level_unvoted_proposer_block: u32,
}

impl Chain {
    /// Create a new voter chain.
    pub fn new(best_block: H256) -> Self {
        let unvoted_proposer_blocks: HashMap<u32, H256> = HashMap::new();
        return Self {
            best_block,
            best_level: 0,
            unvoted_proposer_blocks,
            min_level_unvoted_proposer_block: 1,
            max_level_unvoted_proposer_block: 0, // TODO: 1,0!!
        };
    }

    /// Add a voter block to the chain and get the result of the insertion.
    pub fn add_voter_block(
        &mut self,
        block: H256,
        block_parent: H256,
        level: u32,
    ) -> NodeUpdateStatus {
        // if the new block is mined on the previous best block
        if self.best_block == block_parent {
            self.best_level += 1;
            self.best_block = block;
            return NodeUpdateStatus::ExtendedMainChain;
        }
        // if the new block attaches to a side chain, and now the side chain becomes as long as the
        // main chain
        else if self.best_block != block_parent && self.best_level == level {
            return NodeUpdateStatus::SideChain;
        }
        // the new block attaches to a side chain
        else if self.best_level >= level {
            return NodeUpdateStatus::SideChain;
        }
        // the block attaches to a side chain, and now the side chain becomes the main chain
        else if self.best_level < level {
            return NodeUpdateStatus::LongerFork;
        } else {
            unreachable!();
        }
    }

    /// Switch the main chain to the given best block and best level.
    pub fn switch_the_main_chain(&mut self, best_block: H256, best_level: u32) {
        self.best_block = best_block;
        self.best_level = best_level;
    }

    /// Mark the given proposer block hash as to be voted at the given proposer level.
    pub fn insert_unvoted(&mut self, level: u32, hash: H256) {
        if level == self.max_level_unvoted_proposer_block + 1 {
            if self.unvoted_proposer_blocks.contains_key(&level) {
                panic!("This should have happened");
            }
            self.unvoted_proposer_blocks.insert(level, hash);
            self.max_level_unvoted_proposer_block += 1;
        } else if level > self.max_level_unvoted_proposer_block + 1 {
            panic!("Trying to mark the proposer block to vote for at level geater than the deepest unvoted level + 1");
        } else {
            // Ignore. This happens when we get another proposer block at level which we have
            // marked a proposer block to vote for.
        }
    }

    /// Remove the proposer block hash to vote for at the given proposer level.
    pub fn remove_unvoted(&mut self, level: u32) {
        if level != self.min_level_unvoted_proposer_block {
            panic!("Trying to remove the proposer block to vote for at level not equal to the smallest unvoted level");
        }
        self.unvoted_proposer_blocks.remove(&level);
        self.min_level_unvoted_proposer_block += 1;
    }

    // TODO: so many panics
    /// Set the proposer blocks to vote for at the given levels. The levels must be continuous and
    /// last until right behind the current smallest unvoted proposer level.
    ///
    /// An example:
    /// ```ignore
    /// Initial State:
    ///     PB proposer_blocks[0].1
    ///     |
    ///     PB
    ///     |
    ///     PB
    ///     |
    ///     PB proposer_blocks[-1].1
    ///     |
    ///     PB (min_level_unvoted_proposer_block)
    ///     |
    ///     PB
    ///     |
    ///     PB
    ///     |
    ///     PB (max_level_unvoted_proposer_block)
    ///
    ///     Final state:
    ///     PB proposer_blocks[0].1 (min_level_unvoted_proposer_block)
    ///     |
    ///     PB
    ///     |
    ///     PB
    ///     |
    ///     PB proposer_blocks[-1].1
    ///     |
    ///     PB
    ///     |
    ///     PB
    ///     |
    ///     PB
    ///     |
    ///     PB (max_level_unvoted_proposer_block)
    /// ```
    pub fn add_unvoted_while_switching(&mut self, left_segment_proposer_votes: Vec<(H256, u32)>) {
        let mut pre_level = left_segment_proposer_votes[0].1;

        self.unvoted_proposer_blocks
            .insert(pre_level, left_segment_proposer_votes[0].0);
        for proposer_block in left_segment_proposer_votes[1..].iter() {
            let level = proposer_block.1;
            if level != pre_level + 1 {
                panic!("The proposer levels provided are not continuous.");
            }
            self.unvoted_proposer_blocks.insert(level, proposer_block.0);
            pre_level = level;
        }

        // set the smallest unvoted proposer level
        if pre_level != self.min_level_unvoted_proposer_block - 1 {
            panic!("The proposer levels provided do not last until the previous smallest unvoted proposer - 1.")
        }
        self.min_level_unvoted_proposer_block = left_segment_proposer_votes[0].1;
    }

    /// Unmark the proposer blocks to vote for at the given proposer levels.
    pub fn remove_unvoted_while_switching(
        &mut self,
        right_segment_proposer_votes: Vec<(H256, u32)>,
    ) {
        for vote in right_segment_proposer_votes.iter() {
            self.remove_unvoted(vote.1);
        }
    }

    /// Return an ordered list of proposer blocks to vote for. This if for mining.
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
            "best_block: {}; best_level: {}",
            self.best_block, self.best_level
        )?; // Ignoring status for now
        Ok(())
    }
}

/// Represent a fork in the voter chain. It is used when a longer fork appears in a voting chain.
///
/// ```ignore
///             B
///             |
///             B---- (common parent)
///             |   |
///             B   B (The segments start on this level)
///             |   |
///             B   B
///             |   |
/// (left leaf) B   B
///                 |
///                 B
///                 |
///                 B (right leaf)
/// ```
pub struct Fork {
    /// The last block that is in common for the two forks.
    pub common_parent: H256,
    /// The blocks that live on the previous main chain (the left segment, the shorter one).
    pub left_segment: Vec<H256>,
    /// The blocks that live on the new main chain (the right segment, the longer one).
    pub right_segment: Vec<H256>,
}
