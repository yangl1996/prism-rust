use std::collections::{HashSet};
use super::utils::*;
use super::proposer_tree::PropNode;


pub struct TxNode<'a>{
    /// Block Id
    block_id : BlockId,
    /// Parent prop block
    parent_prop_block_id: &'a PropNode<'a>,
    /// Prop block which refers this block
    child_prop_block_id: &'a PropNode<'a>,
}

impl<'a> Node for TxNode<'a>{
    fn get_type() -> NodeType{ return NodeType::Transaction }
}



/// Stores all the tx nodes
pub struct TxPool<'a>{
    /// Set of all transaction nodes
    tx_nodes: HashSet<&'a TxNode<'a>>
}

impl<'a> TxPool<'a>{
    /// Initialize Tx pool
//    pub fn new() -> Self{
//        let tx_nodes: HashSet<TxNode> = HashSet::new();
//        return TxPool{tx_nodes};
//    }

    /// Add a tx block
    pub fn add_tx_block(&mut self, node: TxNode<'a>){
//        self.tx_nodes.insert(node); Todo: Define a hash insert ???
    }
}
