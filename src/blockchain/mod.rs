mod transaction;
mod proposer;
mod voter;
use super::block::{Block, Content};
use super::crypto::hash::{Hashable, H256};
use serde::{Serialize, Deserialize};
use proposer::{ProposerNodeData, ProposerTree};
use voter::{VoterNodeData, VoterChain};
use std::collections::HashMap;

use petgraph::{Directed, Undirected, graph::NodeIndex};
use petgraph::graphmap::GraphMap;



#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum NodeData{
    Proposer(ProposerNodeData),
    Voter(VoterNodeData),
}


#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum Edge{
    /// Tx edge types
    TransactionToProposerParent,
    /// Prop edge types
    ProposerToProposerParent,
    ProposerToProposerReference,
    ProposerToTransactionReference,
    ProposerToTransactionLeaderReference,
    /// Voter edge types
    VoterToProposerParent,
    VoterToVoterParent,
    VoterToProposerVote,
}

pub struct BlockChain{
    /// To store the graph structure of Prism
    pub graph: GraphMap<H256, Edge, Undirected>,
    pub proposer_tree: ProposerTree,
    pub voter_chains: Vec<VoterChain>,
    /// Contains extra data about the nodes.
    node_data:HashMap<H256, NodeData>
}

impl BlockChain {
    /// Used when the blockchain starts
    pub fn new(number_of_voter_chains: u16) -> Self {
        /// Initializing an empty objects
        let mut graph = GraphMap::<H256, Edge, Undirected>::new();
        let mut proposer_tree = ProposerTree::default();
        let mut voter_chains: Vec<VoterChain> = vec![];
        let mut node_data = HashMap::<H256, NodeData>::new();

        /// 1. Proposer genesis block
        /// 1a. Adding proposer genesis block in the graph
        let proposer_genesis_node = ProposerNodeData::genesis(number_of_voter_chains);
        let proposer_hash_vec: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]; /// Hash vector of proposer genesis block. todo: Shift to a global config  file
        graph.add_node((&proposer_hash_vec).into());
        /// Adding node data of proposer genesis block in the hashmap
        node_data.insert((&proposer_hash_vec).into(), NodeData::Proposer(proposer_genesis_node));
        // 1b. Initializing proposer tree
        proposer_tree.best_block= (&proposer_hash_vec).into();
        proposer_tree.prop_nodes.push(vec![(&proposer_hash_vec).into()]);
        proposer_tree.leader_nodes.push(Some((&proposer_hash_vec).into()));

        /// 2. Voter geneses blocks
        for chain_number in 0..(number_of_voter_chains) {
            /// 2a. Adding voter chain i genesis block in the graph
            let voter_genesis_node = VoterNodeData::genesis(chain_number as u16);
            let b1 = ((chain_number+1) >> 8) as u8;
            let b2 = (chain_number+1) as u8;
            let voter_hash_vec: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,b1,b2];/// Hash vector of voter genesis block. todo: Shift to a global config  file
            graph.add_node((&voter_hash_vec).into());
            /// Adding node data in the hashmap
            node_data.insert((&voter_hash_vec).into(), NodeData::Voter(voter_genesis_node));
            /// 2b. Initializing a Voter chain
            let voter_chain = VoterChain::new(chain_number, (&voter_hash_vec).into());
            voter_chains.push(voter_chain);
        }
        return Self{graph, proposer_tree, voter_chains, node_data};
    }

    //todo: Add a restoration function. This requires DB.

    /// Add a new block to the graph. This function is called when a new block is received. We assume that all the referred block are available.
    pub fn add_block_as_node(&mut self, block: Block) {
        let block_hash = block.hash();
        let parent_proposer_block_hash = block.header.parent_hash;
        /// Add the node to the graph
        self.graph.add_node(block_hash);
        /// Use the content of the block to add the edges.
        let content: &Content = &block.content;
        match content {

            Content::Transaction(_) => {
                /// Adding edge from tx block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::TransactionToProposerParent);
            },

            Content::Proposer(content) => {
                /// 1, Adding edge from prop block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::ProposerToProposerParent);

                /// 2. Iterating through the list of proposer blocks referred in the content of the given proposer block
                content.proposer_block_hashes.iter().map(
                    |&x| self.graph.add_edge(block_hash, x, Edge::ProposerToProposerReference));
                /// 3. Iterating through the list of transaction blocks referred in the content of the given proposer block
                content.transaction_block_hashes.iter().map(
                    |&x| self.graph.add_edge(block_hash, x, Edge::ProposerToTransactionReference));

                /// 4. Creating proposer node data and updating the level.
                let mut proposer_node_data = ProposerNodeData::default();
                let proposer_parent_node_data :&NodeData = self.get_node_data(&parent_proposer_block_hash);
                if let NodeData::Proposer(parent_node) = proposer_parent_node_data {
                    proposer_node_data.level = parent_node.level + 1
                }
                else {panic!("The node data in a proposer parent is not correct")};

                /// 5. Adding node data
                self.node_data.insert(block_hash, NodeData::Proposer(proposer_node_data));
                /// 6. Adding the block to the proposer tree.
                self.proposer_tree.add_block_at_level(block_hash, proposer_node_data.level);
            },

            Content::Voter(content) => {

                /// 1, Adding edge from voter block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::VoterToProposerParent);
                /// 2. Adding edge from voter block to its voter parent
                self.graph.add_edge(block_hash, content.voter_parent_hash, Edge::VoterToVoterParent);

                for prop_block in content.proposer_block_votes.iter() {
                    /// 3. Adding edge from voter block to proposer votees
                    self.graph.add_edge(block_hash, (*prop_block).clone(), Edge::VoterToProposerVote);
                    /// 4a. Incrementing the votes of the proposer block
                    let proposer_node_data: &NodeData = self.get_node_data(&prop_block);
                    if let NodeData::Proposer(mut node_data) = proposer_node_data {
                        node_data.votes += 1;
                        /// 4b. Adding the vote on proposer tree
                        self.proposer_tree.add_vote_at_level(block_hash, node_data.level);
                    }
                    else { panic!("Wrong!!") }

                }

                /// 5a. Creating voter node data and updating the level of the parent.
                let mut voter_node_data = VoterNodeData::default();
                let parent_voter_node_data :&NodeData = self.get_node_data(&content.voter_parent_hash);
                if let NodeData::Voter(node_data) = parent_voter_node_data {
                    voter_node_data.level = node_data.level + 1;
                    voter_node_data.chain_number = node_data.chain_number;
                    voter_node_data.status = node_data.status
                }
                else {panic!("The node data in a voter parent is not correct")};
                self.node_data.insert(block_hash, NodeData::Voter(voter_node_data));

                /// 6. Updating the voter chain.
                self.voter_chains[voter_node_data.chain_number as usize].update_voter_chain(
                    block_hash, content.voter_parent_hash, voter_node_data.level
                )
            },
        };
    }

    /// Returns the node data using the hashmap and handles errors.
    fn get_node_data(&self, hash : &H256) -> &NodeData{
        let node: Option<&NodeData> = self.node_data.get(hash);
        match node {
            Some(node) => { return node; }
            None => panic!("The parent block is not present in the hashmap"),
        }
    }

    /// Get the best blocks on each voter chain
    pub fn get_voter_parents(&self) -> Vec<H256> {
        let voter_parents: Vec<H256> = self.voter_chains.iter().map(|&x| x.best_block).collect();
        return voter_parents;
    }

    /// Get the best block on proposer tree
    pub fn get_proposer_parent(&self) -> H256 {
        return self.proposer_tree.best_block;
    }
}


#[cfg(test)]
mod tests {
    use crate::crypto::hash::{H256};
    use super::*;
    use rand::{Rng, RngCore};
    
    #[test]
    fn blockchain_initialization(){
        let mut rng = rand::thread_rng();
        let number_of_chains = 100;
        let blockchain = BlockChain::new(number_of_chains);

        /// Checking proposer tree's genesis block hash
        let proposer_genesis_hash_shouldbe: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]; /// Hash vector of proposer genesis block. todo: Shift to a global config  file
        let proposer_genesis_hash_shouldbe: H256 = (&proposer_genesis_hash_shouldbe).into();
        assert_eq!(proposer_genesis_hash_shouldbe, blockchain.proposer_tree.best_block);

        /// Checking all voter tree's genesis block hash
        for chain_number in 0..number_of_chains{
            let b1 = ((chain_number+1) >> 8) as u8;
            let b2 = (chain_number+1) as u8;
            let voter_genesis_hash_shouldbe: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,b1,b2];/// Hash vector of voter genesis block. todo: Shift to a global config  file
            let voter_genesis_hash_shouldbe: H256 = (&voter_genesis_hash_shouldbe).into();
            assert_eq!(voter_genesis_hash_shouldbe, blockchain.voter_chains[chain_number as usize].best_block);
        }
    }

}
