/// A transaction pool.

//use std::collections::{HashSet};
use super::utils::*;
use super::proposer_tree::PropNode;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone, PartialEq)]
pub struct TxNode<'a>{
    /// Block Id
    node_id : BlockId,
    /// Parent prop node
    parent_prop_node: Option<&'a PropNode<'a>>,
    /// Prop node which refers this node
    child_prop_node: Option<&'a PropNode<'a>>,
}

impl<'a> TxNode<'a>{
    fn set_parent(&mut self, parent_prop_node: &'a PropNode<'a>){
        self.parent_prop_node = Some(parent_prop_node);
    }

    /// Add a prop node which is refers 'self'.
    fn add_prop_reference(&mut self, referred_prop_node: &'a PropNode<'a>){
        self.child_prop_node = Some(referred_prop_node);
    }
}

impl<'a> Default for TxNode<'a> {
    fn default() -> Self {
        let node_id = BlockId::default();
        let parent_prop_node: Option<&'a PropNode<'a>> = None;
        let child_prop_node: Option<&'a PropNode<'a>> = None;
        return TxNode {node_id, parent_prop_node, child_prop_node};
    }
}


impl<'a> Node for TxNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Transaction }
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
}
