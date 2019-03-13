mod transaction;
mod proposer;
mod voter;
mod status;
use super::block::{Block, Content};
use super::crypto::hash::{Hashable};
use serde::{Serialize, Deserialize};
use transaction::{TxNode, TxPool};
use proposer::{PropNode, PropTree};
use voter::{VoterNode, VoterChain};


#[derive(Serialize, Clone)]
pub struct BlockChainGraph<'a>{
    pub tx_block_pool: TxPool<'a>,
    pub proposer_tree: PropTree<'a>,
    pub voter_chains: Vec<VoterChain<'a>>
}

/// Maintains the graph which comprises of three types blocks. The goal of this block structure is
/// to confirm a transaction.
impl<'a> BlockChainGraph<'a>{

    /// Used when the blockchain starts
    pub fn new(number_of_voter_chains: u32) -> Self {

        /// Initialize a transaction block pool
        let tx_block_pool = TxPool::new();

        /// Initialize the genesis block of Proposer tree and the proposer tree
        let prop_genesis_block = PropNode::genesis();
        let prop_block_tree = PropTree::new(prop_genesis_block);

        /// Initialize the genesis block of m voter trees and the voter trees.
        let mut voter_chains: Vec<VoterChain> =  vec![];

        for i in 0..number_of_voter_chains{
            let voter_genesis_block = VoterNode::genesis(i as u16);
            let voter_chain = VoterChain::new(voter_genesis_block);
            voter_chains.push(voter_chain);
        }

        return BlockChainGraph{tx_block_pool, proposer_tree: prop_block_tree, voter_chains}
    }

    //todo: Add a restoration function. This requires DB.

    pub fn get_number_of_voter_chains(&self) -> u32{
        return self.voter_chains.len() as u32;
    }

    /// Add a new block to the graph. This function is called when a new block is received.
    pub fn add_block_as_node(&mut self, block: Block){
        let content: &Content = &block.content;
        /// The block type is determined by its content. todo: We could make this faster if required
        match content {
            /// If the block is a transaction block, then its corresponding tx node is added to the graph.
            Content::Transaction(_c) => {
                // todo: add transaction block to the unreferenced tx block pool (global)

                /// Create a default tx node whose edges will be populated later
                let mut tx_node = TxNode::default();
                //// Initializing node id
                tx_node.block_hash = block.hash();
                /// Extracting the reference of proposer block corresponding to the hash of the block's parent
                let mut parent_prop_node :&PropNode = self.proposer_tree.get_prop_node_from_block_hash(&block.header.parent_hash);
                // Adding an (parent) directed edge from  tx_node to the parent_prop_node
                tx_node.parent_prop_node = Some(parent_prop_node);
                // The tx block has not been yet referenced so we dont set its child.

//                parent_prop_node.

//              /// Add the tx block to the tx block pool
//                self.tx_block_pool.add_node(tx_node); todo: Make this work. Lifetime issues
            },

            Content::Proposer(c) => {
                /// Create a default prop node whose edges will be populated later
                let mut prop_node = PropNode::default();
                // Initializing node id
                prop_node.block_hash = block.hash();
                /// Extracting the reference of proposer block corresponding to the hash of the block's parent
                let parent_prop_node :&PropNode = self.proposer_tree.get_prop_node_from_block_hash(&block.header.parent_hash);
                prop_node.parent_prop_node = Some(parent_prop_node);
                // The level of a  proposer block  is  defined as 1 more than its parent's level.
                prop_node.level = parent_prop_node.level + 1;


                /// Iterating through the list of transaction blocks referred in the content of the proposer block
                for tx_block_hash in c.transaction_block_hashes.iter(){
                    /// Extracting the reference of tx block corresponding to tx_block_hash
                    let tx_node_referred :&TxNode = self.tx_block_pool.get_tx_node_from_block_hash(&tx_block_hash);
                    /// Adding an (ref) directed edge from prop_node to the tx_node_referred
                    prop_node.add_tx_reference(tx_node_referred);
//                    prop_node_referred.add_child_node(&prop_node); // todo: Make it work
                }
//                /// Add the prop_node to proposer tree
//                self.proposer_tree.add_node(prop_node); todo: Make this work. Lifetime issues
            },

            Content::Voter(c) => {
                /// Create a default voter node whose edges will be populated later
                let mut voter_node = VoterNode::default();
                // Initialize chain id of the node
                voter_node.chain_number = c.chain_number;
                /// This node will be added tot the voter chain with chain_number c.chain_number
                let voter_chain = &mut self.voter_chains[c.chain_number as usize];
                // Initializing node id
                voter_node.block_hash = block.hash();
                /// Extracting the reference of voter block corresponding to the hash of the block's voter parent
                let parent_voter_node :&VoterNode = voter_chain.get_voter_node_from_block_hash(&c.voter_parent_hash);
                voter_node.parent_voter_node = Some(parent_voter_node);
                /// Extracting the reference of proposer block corresponding to the hash of the block's parent
                let parent_prop_node :&PropNode = self.proposer_tree.get_prop_node_from_block_hash(&block.header.parent_hash);
                voter_node.parent_prop_node = Some(parent_prop_node);
                // The level of a voter block  is  defined as 1 more than its parent's level.
                voter_node.level = parent_voter_node.level + 1;

                /// Iterating through the list of votes referred in the content
                for votes in c.proposer_block_votes.iter(){
                    /// Extracting the reference of tx block corresponding to tx_block_hash
                    let prop_node_voted :&PropNode = self.proposer_tree.get_prop_node_from_block_hash(&votes);
                    /// Adding an (vote) directed edge from voter_node to the prop_node_voted
                    voter_node.add_vote(prop_node_voted);
//                    prop_node_voted.add_vote(&voter_node); // todo: Make it work
                }
//                /// Add the voter_node to voter chain
//                voter_chain.add_node(voter_node); // todo: Make this work. Lifetime issues
            },
        };
    }
}
