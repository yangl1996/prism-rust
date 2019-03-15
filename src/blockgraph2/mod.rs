mod transaction;
mod proposer;
mod voter;
use super::block::{Block, Content};
use super::crypto::hash::{Hashable, H256};
use serde::{Serialize, Deserialize};
use proposer::{Proposer};
use voter::{Voter};
use std::collections::HashMap;

use petgraph::{Directed, Undirected, graph::NodeIndex};
use petgraph::graphmap::GraphMap;



#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum NodeData{
    Proposer(Proposer),
    Voter(Voter),
}


#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum Edge{
    /// Tx edge types
    TransactionToProposerParent,
    /// Prop edge types
    ProposerToProposerParent,
    ProposerToProposerReference,
    ProposerToTransactionReference,
    /// Voter edge types
    VoterToProposerParent,
    VoterToVoterParent,
    VoterToProposerVote,
}

pub struct BlockChain{
    pub graph: GraphMap<H256, Edge, Undirected>,
    pub number_of_voter_chains: u16,
    /// Maintains Extra data about the node.
    node_data_hashmap :HashMap<H256, NodeData>
}

impl BlockChain {
    /// Used when the blockchain starts
    pub fn new(number_of_voter_chains: u16) -> Self {
        /// Initializing an empty graph object
        let mut graph = GraphMap::<H256, Edge, Undirected>::new();
        let mut node_data_hashmap = HashMap::<H256, NodeData>::new();

        /// 1. Proposer genesis block
        /// Adding proposer genesis block in the graph
        let proposer_genesis_node = Proposer::genesis(number_of_voter_chains);
        let proposer_hash_vec: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]; /// Hash vector of proposer genesis block. todo: Shift to a global config  file
        graph.add_node((&proposer_hash_vec).into());
        /// Adding node data of proposer genesis block in the hashmap
        node_data_hashmap.insert((&proposer_hash_vec).into(), NodeData::Proposer(proposer_genesis_node));

        /// 2. Voter geneses blocks
        /// Adding voter chain genesis blocks
        for chain_number in 1..(number_of_voter_chains+1) {
            /// Adding voter chain i genesis block in the graph
            let voter_genesis_node = Voter::genesis(chain_number as u16);
            let b1 = chain_number as u8;
            let b2 = (chain_number >> 8) as u8;
            let voter_hash_vec: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,b1,b2];/// Hash vector of voter genesis block. todo: Shift to a global config  file
            graph.add_node((&voter_hash_vec).into());
            /// Adding node data in the hashmap
            node_data_hashmap.insert((&voter_hash_vec).into(), NodeData::Voter(voter_genesis_node));
        }
        return Self {graph, number_of_voter_chains, node_data_hashmap};
    }


    //todo: Add a restoration function. This requires DB.


    /// Add a new block to the graph. This function is called when a new block is received. We assume that all the referred block are available.
    pub fn add_block_as_node(&mut self, block: Block) {
        /// Add the node to the graph
        let block_hash = block.hash();
        self.graph.add_node(block_hash);
        let parent_proposer_block_hash = block.header.parent_hash;
        /// Use the content of the block to add the edges.
        let content: &Content = &block.content;
        /// The block type is determined by its content. todo: We could make this faster if required
        match content {

            Content::Transaction(_c) => {
                /// Adding edge from tx block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::TransactionToProposerParent);
            },

            Content::Proposer(c) => {
                /// 1, Adding edge from prop block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::ProposerToProposerParent);

                /// 2. Iterating through the list of proposer blocks referred in the content of the given proposer block
                c.proposer_block_hashes.iter().map(
                    |&x| self.graph.add_edge(block_hash, x, Edge::ProposerToProposerReference));
                /// 3. Iterating through the list of transaction blocks referred in the content of the given proposer block
                c.transaction_block_hashes.iter().map(
                    |&x| self.graph.add_edge(block_hash, x, Edge::ProposerToTransactionReference));

                /// 4. Creating proposer node data and updating the level of the parent.
                let mut proposer_node_data = Proposer::default();
                let parent_proposer_node_data :&NodeData = self.get_node_data(&parent_proposer_block_hash);
                if let NodeData::Proposer(node_data) = parent_proposer_node_data {
                    proposer_node_data.level = node_data.level + 1
                }
                else {panic!("The node data in a proposer parent is not correct")};
            },

            Content::Voter(c) => {

                /// 1, Adding edge from voter block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::VoterToProposerParent);
                /// 2. Adding edge from voter block to its voter parent
                self.graph.add_edge(block_hash, c.voter_parent_hash, Edge::VoterToVoterParent);

                for prop_block in c.proposer_block_votes.iter() {
                    /// 3. Adding edge from voter block to proposer votees
                    self.graph.add_edge(block_hash, (*prop_block).clone(), Edge::VoterToProposerVote);
                    /// 4. Incrementing the votes of the proposer block
                    let proposer_node_data: &NodeData = self.get_node_data(&prop_block);
                    if let NodeData::Proposer(mut node_data) = proposer_node_data
                        { node_data.votes += 1 }
                    else { panic!("Not a dog") }
                }

                /// 4. Creating voter node data and updating the level of the parent.
                let mut voter_node_data = Voter::default();
                let parent_voter_node_data :&NodeData = self.get_node_data(&c.voter_parent_hash);
                if let NodeData::Voter(node_data) = parent_voter_node_data {
                    voter_node_data.level = node_data.level + 1;
                    voter_node_data.chain_number = node_data.chain_number;
                    voter_node_data.status = node_data.status
                }
                else {panic!("The node data in a voter parent is not correct")};
            },
        };
    }

    /// Returns the node data using the hashmap and handles errors.
    fn get_node_data(&self, hash : &H256) -> &NodeData{
        let node: Option<&NodeData> = self.node_data_hashmap.get(hash);
        match node {
            Some(node) => { return node; }
            None => panic!("The parent block is not present in the hashmap"),
        }
    }
}
