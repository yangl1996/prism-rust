use crate::block::Block;
use crate::crypto::hash::{Hashable, H256};
use petgraph::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct BlockBuffer {
    blocks: Mutex<HashMap<H256, Block>>,
    dependency_graph: Mutex<DiGraphMap<H256, ()>>,
}

impl BlockBuffer {
    pub fn new() -> Self {
        return Self {
            blocks: Mutex::new(HashMap::new()),
            dependency_graph: Mutex::new(DiGraphMap::new()),
        };
    }

    pub fn insert(&self, block: Block, dependencies: &[H256]) {
        let hash = block.hash();
        let mut blocks = self.blocks.lock().unwrap();
        let mut depgraph = self.dependency_graph.lock().unwrap();

        blocks.insert(hash, block);

        depgraph.add_node(hash);
        for dep_hash in dependencies {
            depgraph.add_node(*dep_hash);
            depgraph.add_edge(hash, *dep_hash, ());
        }

        drop(blocks);
        drop(depgraph);
    }

    pub fn satisfy(&self, hash: H256) -> Vec<Block> {
        let mut blocks = self.blocks.lock().unwrap();
        let mut depgraph = self.dependency_graph.lock().unwrap();

        let mut resolved: Vec<H256> = vec![];
        let mut stack: Vec<H256> = vec![hash];
        while let Some(hash) = stack.pop() {
            // mark this block as resolved in the graph, and iterate through all of its dependends
            // to see whether we can unblock some (check whether we are the only neighbor)
            let dependents = depgraph.neighbors_directed(hash, Incoming);
            for node in dependents {
                if depgraph.edges(node).count() == 1 {
                    stack.push(node);
                }
            }
            if !depgraph.remove_node(hash) {
                continue;
            }
            // add the current block as resolved
            resolved.push(hash);
        }

        let mut resolved_blocks: Vec<Block> = vec![];
        for hash in &resolved {
            let block = match blocks.remove(hash) {
                Some(b) => b,
                None => {
                    // This is possible when hash is of a block that we just received
                    continue;
                }
            };
            resolved_blocks.push(block);
        }

        drop(blocks);
        drop(depgraph);
        return resolved_blocks;
    }
}
