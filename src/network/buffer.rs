use crate::block::Block;
use crate::crypto::hash::{Hashable, H256};
use std::collections::{HashMap, HashSet};

pub struct BlockBuffer {
    /// All blocks that have been received but not processed.
    blocks: HashMap<H256, Block>,
    // TODO: we could use a sorted vector for better performance
    /// Mapping between all blocks that have been received and not processed, and their
    /// dependencies
    dependency: HashMap<H256, HashSet<H256>>,
    /// Mapping between all blocks that have not been processed (but either received or
    /// not), and their dependents
    dependent: HashMap<H256, HashSet<H256>>,
}

impl BlockBuffer {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            dependency: HashMap::new(),
            dependent: HashMap::new(),
        }
    }

    /// Buffer a block whose parent and/or references are missing.
    pub fn insert(&mut self, block: Block, dependencies: &[H256]) {
        // Potential race condition here: if X depends on A. Suppose X is received first and
        // validation finds that we miss block A. Then we need to insert X. However, at this moment
        // A comes. Unaware of X, we just process A without marking X's deps as satisfied. Then X
        // finally gets inserted, and is never popped out of the buffer. The core reason is that
        // validation and buffer insert are not one atomic operation (and we probably don't want
        // to do so). Conclusion: for now we make validation and buffer an atomic operation.
        let hash = block.hash();

        self.blocks.insert(hash, block);

        let mut dependency = HashSet::new();
        for dep_hash in dependencies {
            dependency.insert(*dep_hash);
            if !self.dependent.contains_key(&dep_hash) {
                let dependent = HashSet::new();
                self.dependent.insert(*dep_hash, dependent);
            }
            let dependent = self.dependent.get_mut(&dep_hash).unwrap();
            dependent.insert(hash);
        }
        self.dependency.insert(hash, dependency);
    }

    /// Mark that the given block has been processed.
    pub fn satisfy(&mut self, hash: H256) -> Vec<Block> {
        let mut resolved_blocks: Vec<Block> = vec![];

        // get what blocks are blocked by the block being satisfied
        if let Some(dependents) = self.dependent.remove(&hash) {
            for node in &dependents {
                let dependency = self.dependency.get_mut(&node).unwrap();
                dependency.remove(&hash);
                if dependency.is_empty() {
                    self.dependency.remove(&node).unwrap();
                    resolved_blocks.push(self.blocks.remove(&node).unwrap());
                }
            }
        }
        resolved_blocks
    }
}
