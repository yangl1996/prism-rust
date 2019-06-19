use crate::block::Block;
use crate::crypto::hash::{Hashable, H256};
use petgraph::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct BlockBuffer {
    /// All blocks that have been received but not processed.
    blocks: HashMap<H256, Block>,
    /// All block hashes that we have heard of, but not processed.
    dependency_graph: DiGraphMap<H256, ()>,
}

impl BlockBuffer {
    pub fn new() -> Self {
        return Self {
            blocks: HashMap::new(),
            dependency_graph: DiGraphMap::new(),
        };
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

        self.dependency_graph.add_node(hash);
        for dep_hash in dependencies {
            self.dependency_graph.add_node(*dep_hash);
            self.dependency_graph.add_edge(hash, *dep_hash, ());
        }
    }

    pub fn satisfy(&mut self, hash: H256) -> Vec<Block> {
        let mut stack: Vec<H256> = vec![hash];
        let mut resolved_blocks: Vec<Block> = vec![];

        while let Some(hash) = stack.pop() {
            // mark this block as resolved in the graph, and iterate through all of its dependends
            // to see whether we can unblock some (check whether we are the only neighbor)
            let dependents = self.dependency_graph.neighbors_directed(hash, Incoming);
            for node in dependents {
                if self.dependency_graph.edges(node).count() == 1 && self.blocks.contains_key(&node) {
                    stack.push(node);
                    resolved_blocks.push(self.blocks.remove(&node).unwrap());
                }
            }
            self.dependency_graph.remove_node(hash);
        }
        return resolved_blocks;
    }
}
