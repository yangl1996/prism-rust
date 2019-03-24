/// A transaction pool.

//use std::collections::{HashSet};
use super::status::*;
use super::proposer::PropNode;
use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256};

#[derive(Serialize, Clone, PartialEq)]
pub struct TxNode<'a>{
    /// Block Id
    pub block_hash : H256,
    /// Parent prop node
    pub parent_prop_node: Option<&'a PropNode<'a>>,
    /// Prop node which refers this node
    pub child_prop_node: Option<&'a PropNode<'a>>,
}

impl<'a> Default for TxNode<'a> {
    fn default() -> Self {
        let block_hash = H256::default();
        let parent_prop_node: Option<&'a PropNode<'a>> = None;
        let child_prop_node: Option<&'a PropNode<'a>> = None;
        return TxNode {block_hash, parent_prop_node, child_prop_node};
    }
}

/// Stores all the tx nodes
#[derive(Serialize, Clone)]
pub struct TxPool<'a>{
    /// Set of all transaction nodes
    tx_nodes: Vec<TxNode<'a>>
}
impl<'a> TxPool<'a>{
    /// Initialize Tx pool
    pub fn new() -> Self{
        let tx_nodes: Vec<TxNode<'a>> = vec![];
        return TxPool{tx_nodes};
    }

    /// Add a tx node
    pub fn add_node(&mut self, node: TxNode<'a>){
        self.tx_nodes.push(node);
    }

    /// Returns the tx node for the give node id
    /// todo: To yet implement
    pub fn get_tx_node_from_block_hash(&self, block_hash: &H256 ) -> &TxNode {
        unimplemented!();
    }
}
