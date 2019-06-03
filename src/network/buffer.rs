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

        let mut stack: Vec<H256> = vec![hash];
        let mut resolved_blocks: Vec<Block> = vec![];

        while let Some(hash) = stack.pop() {
            // mark this block as resolved in the graph, and iterate through all of its dependends
            // to see whether we can unblock some (check whether we are the only neighbor)
            let dependents = depgraph.neighbors_directed(hash, Incoming);
            for node in dependents {
                if depgraph.edges(node).count() == 1 && blocks.contains_key(&node) {
                    stack.push(node);
                    resolved_blocks.push(blocks.remove(&node).unwrap());
                }
            }
            depgraph.remove_node(hash);
        }

        drop(blocks);
        drop(depgraph);
        return resolved_blocks;
    }
}
