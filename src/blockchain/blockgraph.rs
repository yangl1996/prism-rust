use super::database::{BlockChainDatabase, EDGE_TYPE_1_CFS, EDGE_TYPE_2_CFS};
use super::edge::Edge;
use crate::crypto::hash::H256;
use bincode::{deserialize, serialize};
use rocksdb::WriteBatch;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct BlockGraph {
    pub db: Arc<Mutex<BlockChainDatabase>>,
    pub edge_count: u32,
}

impl BlockGraph {
    /// Adds an edge between too and from of 'edge_type'
    pub fn add_edge_type_1(&mut self, from: H256, to: H256, edge_type: Edge) {
        let db = self.db.lock().unwrap();
        let from_u8: [u8; 32] = from.into();
        let cf = db.handle.cf_handle(&EDGE_TYPE_1_CFS[&edge_type]).unwrap();
        //Check if the 'from' node has a edges of 'edge_type'
        let serialized = db.handle.get_cf(cf, &from_u8).unwrap();
        match serialized {
            Some(s) => {
                let mut neighbours: Vec<H256> = deserialize(&s).unwrap();
                neighbours.push(to);
                let mut batch = WriteBatch::default();
                // Removing the old entry and adding a new one
                batch.delete_cf(cf, &from_u8);
                batch.put_cf(cf, &from_u8, serialize(&neighbours).unwrap());
                db.handle.write(batch);
            }
            None => {
                let neighbours: Vec<H256> = vec![to];
                db.handle
                    .put_cf(cf, &from_u8, serialize(&neighbours).unwrap());
            }
        }
        self.edge_count += 1;
    }

    pub fn add_edge_type_2(&mut self, from: H256, to: H256, edge_type: Edge, value: u32) {
        let db = self.db.lock().unwrap();
        let from_u8: [u8; 32] = from.into();
        let cf = db.handle.cf_handle(&EDGE_TYPE_2_CFS[&edge_type]).unwrap();
        //Check if the 'from' node has a edges of 'edge_type'
        let serialized = db.handle.get_cf(cf, &from_u8).unwrap();
        match serialized {
            Some(s) => {
                let mut neighbours: Vec<(H256, u32)> = deserialize(&s).unwrap();
                neighbours.push((to, value));
                let mut batch = WriteBatch::default();
                // Removing the old entry and adding a new one
                batch.delete_cf(cf, &from_u8);
                batch.put_cf(cf, &from_u8, serialize(&neighbours).unwrap());
                db.handle.write(batch);
            }
            None => {
                let neighbours: Vec<(H256, u32)> = vec![(to, value)];
                db.handle
                    .put_cf(cf, &from_u8, serialize(&neighbours).unwrap());
            }
        }
        self.edge_count += 1;
    }

    /// Returns neighbours of 'from' from all the edge_types
    pub fn get_neighbours_type_1(&mut self, from: H256, edge_types: Vec<Edge>) -> Vec<H256> {
        let mut neighbours: Vec<H256> = vec![];
        let db = self.db.lock().unwrap();
        let from_u8: [u8; 32] = from.into();
        for edge_type in edge_types {
            let cf = db.handle.cf_handle(&EDGE_TYPE_1_CFS[&edge_type]).unwrap();
            let serialized = db.handle.get_cf(cf, &from_u8).unwrap();
            match serialized {
                Some(s) => {
                    let edge_neighbours: Vec<H256> = deserialize(&s).unwrap();
                    neighbours.extend(edge_neighbours);
                }
                None => {}
            }
        }
        return neighbours;
    }

    /// Returns neighbours of 'from' vertex of edge_type along with edge weights
    pub fn get_neighbours_type_2(&mut self, from: H256, edge_type: Edge) -> Vec<(H256, u32)> {
        let db = self.db.lock().unwrap();
        let from_u8: [u8; 32] = from.into();
        let cf = db.handle.cf_handle(&EDGE_TYPE_2_CFS[&edge_type]).unwrap();
        let serialized = db.handle.get_cf(cf, &from_u8).unwrap();
        match serialized {
            Some(s) => {
                let neighbours: Vec<(H256, u32)> = deserialize(&s).unwrap();
                return neighbours;
            }
            None => {
                let neighbours: Vec<(H256, u32)> = vec![];
                return neighbours;
            }
        }
    }

    // Returns yes if the node has an edge.
    pub fn contains_node(&mut self, node: H256) -> bool {
        let db = self.db.lock().unwrap();
        let node_u8: [u8; 32] = node.into();

        // Iter through all edge_type  edge_types 1
        for (_, cf) in EDGE_TYPE_1_CFS.iter() {
            let cf = db.handle.cf_handle(&cf).unwrap();
            let serialized = db.handle.get_cf(cf, &node_u8).unwrap();
            match serialized {
                Some(_) => {
                    return true;
                }
                None => {}
            }
        }

        // Iter through all edge_type  edge_types 2
        for (_, cf) in EDGE_TYPE_2_CFS.iter() {
            let cf = db.handle.cf_handle(&cf).unwrap();
            let serialized = db.handle.get_cf(cf, &node_u8).unwrap();
            match serialized {
                Some(_) => {
                    return true;
                }
                None => {}
            }
        }
        return false;
    }
}
