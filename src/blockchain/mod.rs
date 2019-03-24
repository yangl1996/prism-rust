mod transaction;
mod proposer;
mod voter;
mod test_util;
mod edge;
use super::block::{Block, Content};
use super::crypto::hash::{Hashable, H256};
use serde::{Serialize, Deserialize};
use edge::Edge;
use proposer::NodeData as ProposerNodeData;
use proposer::Tree as ProposerTree;
use voter::NodeData as VoterNodeData;
use voter::Chain as VoterChain;
use transaction::Pool as TxPool;
use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp;

use petgraph::{Directed, Undirected, graph::NodeIndex};
use petgraph::graphmap::GraphMap;

const NUM_VOTER_CHAINS: u16 = 10; // DONT CHANGE THIS

pub struct BlockChain{
    /// Store the three graph structures of Prism
    pub graph: GraphMap<H256, Edge, Directed>,
    pub proposer_tree: ProposerTree,
    pub voter_chains: Vec<VoterChain>,
    pub unconfirmed_transaction_blocks: TxPool,
    /// Contains data about the proposer nodes.
    proposer_node_data_map: HashMap<H256, ProposerNodeData>,
    /// Contains data about the voter nodes.
    voter_node_data_map: HashMap<H256, VoterNodeData>
}

/// Functions to edit the blockchain
impl BlockChain {
    /// Used when the blockchain starts
    pub fn new() -> Self {
        /// Initializing an empty objects
        let mut graph = GraphMap::<H256, Edge, Directed>::new();
        let mut proposer_tree = ProposerTree::default();
        let mut voter_chains: Vec<VoterChain> = vec![];
        let unconfirmed_transaction_blocks: TxPool = TxPool::new();
        let mut proposer_node_data_map = HashMap::<H256, ProposerNodeData>::new();
        let mut voter_node_data_map = HashMap::<H256, VoterNodeData>::new();

        /// 1. Proposer genesis block
        /// 1a. Add proposer genesis block in the graph
        let proposer_genesis_node = ProposerNodeData::genesis(NUM_VOTER_CHAINS);
        let proposer_hash_vec: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; /// Hash vector of proposer genesis block. todo: Shift to a global config  file
        graph.add_node((&proposer_hash_vec).into());
        /// Add node data of proposer genesis block in the hashmap
        proposer_node_data_map.insert((&proposer_hash_vec).into(), proposer_genesis_node);
        // 1b. Initializing proposer tree
        proposer_tree.best_block = (&proposer_hash_vec).into();
        proposer_tree.prop_nodes.push(vec![(&proposer_hash_vec).into()]);
        proposer_tree.leader_nodes.insert(0, (&proposer_hash_vec).into()); // The leader block at level 0

        /// 2. Voter geneses blocks
        for chain_number in 0..(NUM_VOTER_CHAINS) {
            /// 2a. Add voter chain i genesis block in the graph
            let voter_genesis_node = VoterNodeData::genesis(chain_number as u16);
            let b1 = ((chain_number + 1) >> 8) as u8;
            let b2 = (chain_number + 1) as u8;
            let voter_hash_vec: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b1, b2]; /// Hash vector of voter genesis block. todo: Shift to a global config  file
            let voter_hash: H256 = (&voter_hash_vec).into();
            graph.add_node(voter_hash);
            /// Add node data in the hashmap
            voter_node_data_map.insert(voter_hash, voter_genesis_node);
            /// 2b. Initializing a Voter chain
            let voter_chain = VoterChain::new(chain_number, voter_hash);
            voter_chains.push(voter_chain);
            proposer_tree.add_vote_at_level(voter_hash, 0);
        }
        return Self {
            graph,
            proposer_tree,
            voter_chains,
            unconfirmed_transaction_blocks,
            proposer_node_data_map,
            voter_node_data_map
        };
    }

    //todo: Add a restoration function. This requires DB.

    /// Adds directed edges: from->to and to->from with different types.
    fn insert_edge(&mut self, from: H256, to: H256, edge_type: Edge) {
        self.graph.add_edge(to, from, edge_type.reverse_edge());
        self.graph.add_edge(from, to, edge_type);
    }

    /// Add a new block to the graph. This function is called when a new block is received. We assume that all the referred block are available.
    pub fn insert_node(&mut self, block: &Block) {
        let block_hash = block.hash();
        let parent_proposer_block_hash = block.header.parent_hash;
        /// Add the node to the graph
        self.graph.add_node(block_hash);
        /// Use the content of the block to add the edges.
        let content: &Content = &block.content;
        match content {
            Content::Transaction(_) => {
                /// Add edge from tx block to its proposer parent
                self.insert_edge(block_hash, parent_proposer_block_hash, Edge::TransactionToProposerParent);
                self.unconfirmed_transaction_blocks.insert_unconfirmed(block_hash);
            },

            Content::Proposer(content) => {
                /// 1, Add edge from prop block to its proposer parent
                self.insert_edge(block_hash, parent_proposer_block_hash, Edge::ProposerToProposerParent);

                /// 2. Iterate through the list of proposer blocks referred in the content of the given proposer block
                for (position, prop_hash) in content.proposer_block_hashes.iter().enumerate() {
                    self.insert_edge(block_hash, *prop_hash, Edge::ProposerToProposerReference(position as u32));
                }
                println!("Number of tx blocks referred {}", content.transaction_block_hashes.len());

                /// 3. Iterate through the list of transaction block hashes referred in the content of the given proposer block
                for (position, tx_hash) in content.transaction_block_hashes.iter().enumerate() {
                    self.insert_edge(block_hash, *tx_hash, Edge::ProposerToTransactionReference(position as u32));
                }

                /// 4. Creating proposer node data.
                let proposer_parent_node_data: ProposerNodeData = self.proposer_node_data_map[&parent_proposer_block_hash];
                let mut proposer_node_data = ProposerNodeData::default();
                proposer_node_data.level = proposer_parent_node_data.level + 1;

                /// 5. Add node data in the map
                self.proposer_node_data_map.insert(block_hash, proposer_node_data);

                /// 6. Add the block to the proposer tree.
                self.proposer_tree.add_block_at_level(block_hash, proposer_node_data.level);
            },

            Content::Voter(content) => {
                /// 1, Add edge from voter block to its proposer parent
                self.insert_edge(block_hash, parent_proposer_block_hash, Edge::VoterToProposerParent);

                /// 2. Add edge from voter block to its voter parent
                self.insert_edge(block_hash, content.voter_parent_hash, Edge::VoterToVoterParent);

                /// 3. Add edge from voter block to proposer votees
                for prop_block_hash in content.proposer_block_votes.iter() {
                    if self.graph.contains_edge(block_hash, *prop_block_hash) { // if prop_block_hash is parent_proposer_hash too
                        self.insert_edge(block_hash, (*prop_block_hash).clone(), Edge::VoterToProposerParentAndVote);
                    } else {
                        self.insert_edge(block_hash, (*prop_block_hash).clone(), Edge::VoterToProposerVote);
                    }

                    /// 4 Incrementing the votes of the proposer block
                    let ref mut proposer_node_data = self.proposer_node_data_map.get_mut(&prop_block_hash).unwrap();
                    proposer_node_data.votes += 1;
                    self.proposer_tree.add_vote_at_level(block_hash, proposer_node_data.level);
                }

                /// 5a. Creating voter node data and updating the level of the parent.
                let mut voter_node_data = VoterNodeData::default();
                let parent_voter_node_data: VoterNodeData = self.voter_node_data_map[&content.voter_parent_hash];
                voter_node_data.level = parent_voter_node_data.level + 1;
                voter_node_data.chain_number = parent_voter_node_data.chain_number;
                voter_node_data.status = parent_voter_node_data.status;
                self.voter_node_data_map.insert(block_hash, voter_node_data);

                /// 6. Updating the voter chain.
                self.voter_chains[voter_node_data.chain_number as usize].update_voter_chain(
                    block_hash, content.voter_parent_hash, voter_node_data.level
                )
            },
        };
    }
}

/// Functions to infer the blockchain
impl BlockChain {
    /// Return the best blocks on each voter chain. Used for mining.
    pub fn get_voter_best_blocks(&self) -> Vec<H256> {
        let voter_parents: Vec<H256> = self.voter_chains.iter().map(|&x| x.best_block).collect();
        return voter_parents;
    }

    /// Return the best block on proposer tree. Used for mining.
    pub fn get_proposer_best_block(&self) -> H256 {
        return self.proposer_tree.best_block;
    }

    /// Returns the tx blocks directly referred by the proposer block
    pub fn get_referred_tx_blocks(&self, block_hash: H256) -> Vec<H256> {
        if !self.proposer_node_data_map.contains_key(&block_hash) { panic!("The proposer block with hash {} doesn't exist", block_hash); }
        let referred_tx_blocks_edges = self.graph.edges(block_hash).filter(
            |&x|
            if let Edge::ProposerToTransactionReference(_) = *x.2 {
                true
            } else{
            false
            }
        );
        let referred_tx_blocks_nodes: Vec<H256> = referred_tx_blocks_edges.map(|x| x.1).collect();
        return referred_tx_blocks_nodes;
    }

    /// Returns the prop blocks directly referred by the proposer block
    pub fn get_referred_prop_blocks(&self, block_hash: H256) -> Vec<H256> {
        if !self.proposer_node_data_map.contains_key(&block_hash) { panic!("The proposer block with hash {} doesn't exist", block_hash); }
        let referred_prop_blocks_edges = self.graph.edges(block_hash).filter(
            |&x|
                if let Edge::ProposerToProposerReference(_) = *x.2 {
                    true
                } else{
                    false
                }
        );
        let referred_prop_blocks_nodes: Vec<H256> = referred_prop_blocks_edges.map(|x| x.1).collect();
        return referred_prop_blocks_nodes;
    }

    /// Next few functions are used in confirmation policy of the protocol
    /// Return the voter blocks on longest voter chain i
    pub fn get_longest_chain(&self, chain_number: u16) -> Vec<H256> {
        let best_level = self.voter_chains[chain_number as usize].best_level;
        let mut longest_chain: Vec<H256> = vec![];
        let mut top_block: H256 = self.voter_chains[chain_number as usize].best_block;

        /// Recursively push the top block
        for _ in 0..best_level {
            longest_chain.push(top_block);
            top_block = self.get_voter_parent(top_block);
        }
        longest_chain.push(top_block);
        longest_chain.reverse();
        return longest_chain;
    }

    /// Returns votes (prop block hashes) from a chain.
    pub fn get_votes_from_chain(&self, chain_number: u16) -> Vec<H256> {
        let longest_chain: Vec<H256> = self.get_longest_chain(chain_number);
        let mut votes: Vec<H256> = vec![];
        for voter in longest_chain {
            let mut voter_votes = self.get_votes_by_voter(voter);
            voter_votes.reverse();
            votes.extend(voter_votes);
        }

        return votes;
    }

    /// Returns the (proposer) votes of a voter block
    pub fn get_votes_by_voter(&self, block_hash: H256) -> Vec<H256> {
        if !self.voter_node_data_map.contains_key(&block_hash) { panic!("The voter block with hash {} doesn't exist", block_hash); }
        let voter_ref_edges = self.graph.edges(block_hash).filter(|&x| *x.2 == Edge::VoterToProposerVote || *x.2 == Edge::VoterToProposerParentAndVote);
        let voter_ref_nodes: Vec<H256> = voter_ref_edges.map(|x| x.1).collect();
        return voter_ref_nodes;
    }

    /// Return  depths of voters of the given proposer block
    pub fn get_vote_depths_on_proposer(&self, block_hash: H256) -> Vec<u32> {
        if !self.proposer_node_data_map.contains_key(&block_hash) { panic!("The proposer block with hash {} doesn't exist", block_hash); }
        let voter_ref_edges = self.graph.edges(block_hash).filter(|&x| *x.2 == Edge::VoterFromProposerVote || *x.2 == Edge::VoterFromProposerParentAndVote);
        let mut voter_ref_nodes: Vec<u32> = vec![];
        for edge in voter_ref_edges {
            let voter_block_hash = edge.1;
            let voter_node_data = self.voter_node_data_map[&voter_block_hash];
            if !voter_node_data.is_on_longest_chain() { continue; }
            let voter_level = voter_node_data.level;
            let voter_chain_number = voter_node_data.chain_number;
            let voter_chain_depth = self.voter_chains[voter_chain_number as usize].best_level;
            voter_ref_nodes.push(voter_chain_depth - voter_level);
        }
        return voter_ref_nodes;
    }
}

/// Functions to generate the ledger. This uses the confirmation logic of Prism.
impl BlockChain {
    /// todo: Implement this properly (syntax and logic)
    fn lcb_and_ucb_from_vote_depths(&self, votes: Vec<u32>) -> (f32, f32) {
        let answer: f32 = votes.len() as f32;
        return (answer, answer); //todo: Apply the confirmation logic from the paper
    }

    /// Try confirming a leader block at the given level.
    /// If the leader block is first time confirmed then the ledger is updated for that level.
    fn try_confirming_leader_block_at_level(&mut self, level: u32) -> Option<H256> {
        // Return if the level already has leader block.
        if self.proposer_tree.leader_nodes.contains_key(&level) {
            return Some(self.proposer_tree.leader_nodes[&level]);
        }
        let proposers_blocks: &Vec<H256> = &self.proposer_tree.prop_nodes[level as usize];
        let mut lcb_proposer_votes: Vec<f32> = vec![];
        let mut ucb_proposer_votes: Vec<f32> = vec![];
        let mut max_lcb_vote: f32 = -1.0;
        let mut max_lcb_vote_index: usize = 0;

        /// todo: This seems inefficient. Also equal vote situation is not considered.
        for (index, proposer) in proposers_blocks.iter().enumerate() {
            let proposer_votes: Vec<u32> = self.get_vote_depths_on_proposer(*proposer);
            let (lcb, ucb) = self.lcb_and_ucb_from_vote_depths(proposer_votes);
            lcb_proposer_votes.push(lcb);
            ucb_proposer_votes.push(ucb);
            if max_lcb_vote < lcb {
                max_lcb_vote = ucb;
                max_lcb_vote_index = index;
            }
        }

        for index in 0..proposers_blocks.len() {
            if index == max_lcb_vote_index { continue }
            if ucb_proposer_votes[index] > max_lcb_vote {
                return None;
            }
        }

        self.proposer_tree.max_leader_level = cmp::max(self.proposer_tree.max_leader_level, level);

        // Update the continuous_leader_level to level L s.t all levels upto L has a leader block
        for l in level..=self.proposer_tree.max_leader_level{
            if self.proposer_tree.leader_nodes.contains_key(&l) {
                self.proposer_tree.continuous_leader_level = l;
                // todo: Extend the ledger.
            } else {
                break;
            }
        }
        return Some(proposers_blocks[max_lcb_vote_index]);
    }

    /// Return a single leader block at the given level
    pub fn get_leader_block_at_level(&mut self, level: u32) -> Option<H256> {
        if self.proposer_tree.best_level >= level {
            if self.proposer_tree.leader_nodes.contains_key(&level) {
                return Some(self.proposer_tree.leader_nodes[&level]);
            } else {
                let leader_block_option: Option<H256> = self.try_confirming_leader_block_at_level(level);
                if let Some(leader_block) = leader_block_option {
                    self.proposer_tree.leader_nodes.insert(level, leader_block);
                    return Some(leader_block);
                }
            }
        }
        // No proposer block has been yet proposed at this level.
        return None;
    }

    /// Returns the leader blocks from 0 to best level of the proposer tree
    pub fn get_leader_block_sequence(&mut self) -> Vec<Option<H256>> {
        let best_prop_level = self.proposer_tree.best_level;

        let leader_blocks: Vec<Option<H256>> = (1..=best_prop_level).map(
            |level| self.get_leader_block_at_level(level)
        ).collect();
        return leader_blocks;
    }

    /// Return the ledger generated by the leader blocks
    pub fn get_ledger(&self) {
        let ordered_transaction_blocks: Vec<H256> = vec![];
    }
}

/// Functions for mining
impl BlockChain {
    /// Next few functions use the edge information to extract relevant blocks.
    /// Return the proposer parent of the block
    pub fn get_proposer_parent(&self, block_hash: H256) -> H256 {
        let proposer_parent_edges = self.graph.edges(block_hash).filter( |&x|
            (   *x.2 == Edge::TransactionToProposerParent || *x.2 == Edge::ProposerToProposerParent
                || *x.2 == Edge::VoterToProposerParent || *x.2 == Edge::VoterToProposerParentAndVote ));
        let proposer_parent_nodes: Vec<H256> = proposer_parent_edges.map( |x| x.1 ).collect();
        if  proposer_parent_nodes.len() == 1 { return proposer_parent_nodes[0];}
        else {panic!("{} proposer parents for {}", proposer_parent_nodes.len(), block_hash)}
    }

    /// Return the voter parent of a voter block
    pub fn get_voter_parent(&self, block_hash: H256) -> H256 {
        if !self.voter_node_data_map.contains_key(&block_hash) { panic!("The voter block with hash {} doesn't exist", block_hash);}
        let voter_parent_edges = self.graph.edges(block_hash).filter(|&x| *x.2 == Edge::VoterToVoterParent);
        let voter_parent_nodes: Vec<H256> = voter_parent_edges.map( |x| x.1 ).collect();
        if  voter_parent_nodes.len() == 1 { return voter_parent_nodes[0];}
        else {panic!("{} proposer parents for {}", voter_parent_nodes.len(), block_hash)}
    }
}


#[cfg(test)]
mod tests {
    use crate::crypto::hash::{H256};
    use super::*;
    use crate::block::generator as block_generator;
    use crate::block::{Block};
    use rand::{Rng, RngCore};
    use super::test_util;
    use std::fs;

    // At initialization the blockchain only consists of (m+1) genesis blocks.
    // The hash of these genesis nodes in the blockchain graph are fixed for now
    // because we have designed the genesis blocks themselves.
    #[test]
    fn blockchain_initialization(){
        /// Initialize a blockchain with 10  voter chains.
        let blockchain = BlockChain::new();

        /// Checking proposer tree's genesis block hash
        let proposer_genesis_hash_shouldbe: [u8; 32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]; /// Hash vector of proposer genesis block. todo: Shift to a global config  file
        let proposer_genesis_hash_shouldbe: H256 = (&proposer_genesis_hash_shouldbe).into();
        assert_eq!(proposer_genesis_hash_shouldbe, blockchain.proposer_tree.best_block);

        /// Checking all voter tree's genesis block hashes
        for chain_number in 0..NUM_VOTER_CHAINS{
            let b1 = ((chain_number+1) >> 8) as u8;
            let b2 = (chain_number+1) as u8;
            let voter_genesis_hash_shouldbe: [u8; 32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,b1,b2];/// Hash vector of voter genesis block. todo: Shift to a global config  file
            let voter_genesis_hash_shouldbe: H256 = (&voter_genesis_hash_shouldbe).into();
            assert_eq!(voter_genesis_hash_shouldbe, blockchain.voter_chains[chain_number as usize].best_block);
        }
    }

    #[test]
    fn blockchain_growing(){
        let mut rng = rand::thread_rng();
        /// Initialize a blockchain with 10 voter chains.
        let mut blockchain = BlockChain::new();

        /// Store the parent blocks to mine on voter trees.
        let mut voter_best_blocks: Vec<H256> = (0..NUM_VOTER_CHAINS).map( |i| blockchain.voter_chains[i as usize].best_block).collect();// Currently the voter genesis blocks.

        /// Maintains the list of tx blocks.
        let mut tx_block_vec: Vec<Block> = vec![];
        let mut unreferred_tx_block_index = 0;


        println!("\nStep 1:   Initialized blockchain");
        assert_eq!(11, blockchain.graph.node_count(), "Expecting 11 nodes corresponding to 11 genesis blocks");
        assert_eq!(0, blockchain.graph.edge_count(), "Expecting 0 edges");
        println!("Result 1: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());



        println!("\nStep 2:   Added 5 tx blocks on prop genesis");
        /// Mine 5 tx block's with prop_best_block as the parent
        let tx_block_5: Vec<Block> = test_util::tx_blocks_with_parent_hash(5, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_5.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..5{ blockchain.insert_node(&&tx_block_vec[i]); }
        assert_eq!(16, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks");
        assert_eq!(10, blockchain.graph.edge_count(), "Expecting 10 edges");
        println!("Result 2: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());


        println!("\nStep 3:   Added prop block referring these 5 tx blocks");
        /// Generate a proposer block with prop_parent_block as the parent which referencing the above 5 tx blocks
        let prop_block1 = test_util::prop_block(blockchain.proposer_tree.best_block,
   tx_block_vec[0..5].iter().map( |x| x.hash()).collect(), vec![]);
        /// Add the prop_block
        blockchain.insert_node(&&prop_block1);
        assert_eq!(prop_block1.hash(), blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(17, blockchain.graph.node_count(), "Expecting 17 nodes");
        assert_eq!(22, blockchain.graph.edge_count(), "Expecting 22 edges");
        println!("Result 3: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());


        println!("\nStep 4:   Add 10 voter blocks voting on proposer block at level 1");
        for i in 0..NUM_VOTER_CHAINS{
            let voter_block = test_util::voter_block(blockchain.proposer_tree.best_block,
        i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block1.hash()] );
            blockchain.insert_node(&&voter_block);
        }
        assert_eq!(27, blockchain.graph.node_count());
        let prop_block1_votes = blockchain.proposer_node_data_map[&prop_block1.hash()].votes;
        assert_eq!(62, blockchain.graph.edge_count());
        assert_eq!(10, prop_block1_votes, "prop block 1 should have 10 votes" );
        println!("Result 4: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());
//        blockchain = print_edges(blockchain);


        println!("\nStep 5:   Mining 5 tx blocks, 2 prop blocks at level 2 with 3, 5 tx refs");
        unreferred_tx_block_index += 5;
        let tx_block_5: Vec<Block> = test_util::tx_blocks_with_parent_hash(5, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_5.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..5{ blockchain.insert_node(&&tx_block_vec[unreferred_tx_block_index+i]); }
        let prop_block2a = test_util::prop_block(blockchain.proposer_tree.best_block,
        tx_block_vec[5..8].iter().map( |x| x.hash()).collect(), vec![]); // Referring 3 tx blocks
        blockchain.insert_node(&&prop_block2a);
        assert_eq!(prop_block2a.hash(), blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(33, blockchain.graph.node_count(), "Expecting 33 nodes");
        assert_eq!(80, blockchain.graph.edge_count(), "Expecting 80 edges");

        let prop_block2b = test_util::prop_block(prop_block1.hash(),
        tx_block_vec[5..10].iter().map( |x| x.hash()).collect(), vec![]);// Referring 5 tx blocks
        blockchain.insert_node(&&prop_block2b);
        assert_ne!(prop_block2b.hash(), blockchain.proposer_tree.best_block, "prop 2b is not best block");
        assert_eq!(34, blockchain.graph.node_count(), "Expecting 34 nodes" );
        assert_eq!(92, blockchain.graph.edge_count(), "Expecting 92 edges");
        println!("Result 5: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 6:   Add 7+3 votes on proposer blocks at level 2");
        for i in 0..7{
            let voter_block = test_util::voter_block(prop_block2a.hash(),
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block2a.hash()] );
            blockchain.insert_node(&&voter_block);
        }
        for i in 7..10{
            let voter_block = test_util::voter_block(prop_block2b.hash(),
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block2b.hash()] );
            blockchain.insert_node(&&voter_block);
        }
        let prop_block2a_votes = blockchain.proposer_node_data_map[&prop_block2a.hash()].votes;
        let prop_block2b_votes = blockchain.proposer_node_data_map[&prop_block2b.hash()].votes;
        assert_eq!(7, prop_block2a_votes, "prop block 2a should have 7 votes" );
        assert_eq!(3, prop_block2b_votes, "prop block 2b should have 3 votes" );
        assert_eq!(10, blockchain.proposer_tree.all_votes[1].len(), "Level 2 total votes should have 10",);
        assert_eq!(44, blockchain.graph.node_count(), "Expecting 44 nodes");
        assert_eq!(132, blockchain.graph.edge_count(), "Expecting 132 edges");
        println!("Result 6: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 7:   Mining 4 tx block and 1 prop block referring 4 tx blocks + prop_block_2b)");
        unreferred_tx_block_index += 5;
        let tx_block_4: Vec<Block> = test_util::tx_blocks_with_parent_hash(4, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_4.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..4{ blockchain.insert_node(&&tx_block_vec[unreferred_tx_block_index+i]); }
        let prop_block3 = test_util::prop_block(blockchain.proposer_tree.best_block,
        tx_block_vec[10..14].iter().map( |x| x.hash()).collect(), vec![prop_block2b.hash()]); // Referring 4 tx blocks + 1 prop_block
        blockchain.insert_node(&&prop_block3);
        assert_eq!(prop_block3.hash(), blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(49, blockchain.graph.node_count(), "Expecting 49 nodes");
        assert_eq!(152, blockchain.graph.edge_count(), "Expecting 152 edges");
        println!("Result 7: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());


        println!("\nStep 8:   Mining only 3+3 voter blocks voting on none + prob_block3");
        for i in 0..3{
            let voter_block = test_util::voter_block(prop_block2a.hash(), // Mining on 2a (because 3 hasnt showed up yet)
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![] );
            blockchain.insert_node(&&voter_block);
        }
        for i in 3..6{
            let voter_block = test_util::voter_block(prop_block3.hash(), // Mining on 3 after it showed up
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block3.hash()] );
            blockchain.insert_node(&&voter_block);
        }
        assert_eq!(55, blockchain.graph.node_count(), "Expecting 55 nodes");
        assert_eq!(176, blockchain.graph.edge_count(), "Expecting 176 edges");
        println!("Result 8: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 9:   Mining 2 tx block and 1 prop block referring the 2 tx blocks");
        unreferred_tx_block_index += 4;
        let tx_block_2: Vec<Block> = test_util::tx_blocks_with_parent_hash(2, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_2.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..2{ blockchain.insert_node(&&tx_block_vec[unreferred_tx_block_index+i]); }
        let prop_block4 = test_util::prop_block(blockchain.proposer_tree.best_block,
        tx_block_vec[14..16].iter().map( |x| x.hash()).collect(), vec![]); // Referring 4 tx blocks + 1 prop_block
        blockchain.insert_node(&&prop_block4);
        assert_eq!(prop_block4.hash(), blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(58, blockchain.graph.node_count(), "Expecting 58 nodes");
        assert_eq!(186, blockchain.graph.edge_count(), "Expecting 186 edges");
        println!("Result 9: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 10:  1-6 voter chains vote on prop4 and 6-10 voter blocks vote on prop3 and prop4" );
        ///Storing voter_parents used in step 12 test
        for i in 0..10{
            voter_best_blocks[i] = blockchain.voter_chains[i as usize].best_block.clone();
        }
        for i in 0..3{
            let voter_block = test_util::voter_block(prop_block4.hash(), // Mining on 2a (because 3 hasnt showed up yet)
             i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block3.hash(), prop_block4.hash()] );
            blockchain.insert_node(&&voter_block);
        }

        for i in 3..6{
            let voter_block = test_util::voter_block(prop_block4.hash(), // Mining on 2a (because 3 hasnt showed up yet)
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block4.hash()] );
            blockchain.insert_node(&&voter_block);
        }
        for i in 6..10{
            let voter_block = test_util::voter_block(prop_block4.hash(), // Mining on 3 after it showed up
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block3.hash(), prop_block4.hash()] );
            blockchain.insert_node(&&voter_block);
        }
        assert_eq!(68, blockchain.graph.node_count(), "Expecting 68 nodes");
        assert_eq!(240, blockchain.graph.edge_count(), "Expecting 240 edges");
        println!("Result 10:Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        /// Checking the voter chain growth
        for i in 0..6{
            assert_eq!(4, blockchain.voter_chains[i as usize].best_level);
        }
        for i in 6..10{
            assert_eq!(3, blockchain.voter_chains[i as usize].best_level);
        }

        println!("\nStep 11:  Checking get_proposer_parent()");
        assert_eq!(blockchain.get_proposer_parent(prop_block4.hash()), prop_block3.hash());
        assert_eq!(blockchain.get_proposer_parent(tx_block_vec[14].hash()), prop_block3.hash());
        for i in 0..10{
            assert_eq!(blockchain.get_proposer_parent(blockchain.voter_chains[i as usize].best_block), prop_block4.hash());
        }


        println!("Step 12:  Checking get_voter_parent()");
        for i in 0..10{
            assert_eq!(blockchain.get_voter_parent(blockchain.voter_chains[i as usize].best_block), voter_best_blocks[i]);
        }

        println!("Step 13:  Checking get_votes_by_voter()");
        for i in 0..6{
            let votes = blockchain.get_votes_by_voter(blockchain.voter_chains[i as usize].best_block);
            let expected = vec![prop_block4.hash()];
            let matching = votes.iter().zip(expected.iter()).filter(|&(a, b)| a == b).count();
            assert_eq!(matching, expected.len());
        }
        
        for i in 6..10{
            let mut votes = blockchain.get_votes_by_voter(blockchain.voter_chains[i as usize].best_block);
            let mut expected = vec![prop_block3.hash(), prop_block4.hash()];
            votes.sort_by(|a, b| a.partial_cmp(b).unwrap());
            expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let matching = votes.iter().zip(expected.iter()).filter(|&(a, b)| a == b).count();
            assert_eq!(matching, expected.len());
        }

        println!("Step 14:  Checking get_referred_tx_blocks()");
        let mut referred_tx_blocks: Vec<H256> = blockchain.get_referred_tx_blocks(prop_block4.hash());
        let mut expected: Vec<H256> = tx_block_vec[14..16].iter().map( |x| x.hash()).collect();
        referred_tx_blocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let matching = referred_tx_blocks.iter().zip(expected.iter()).filter(|&(a, b)| a == b).count();
        assert_eq!(matching, expected.len());
        // Check 2
        let mut referred_tx_blocks: Vec<H256> = blockchain.get_referred_tx_blocks(prop_block2a.hash());
        let mut expected: Vec<H256> = tx_block_vec[5..8].iter().map( |x| x.hash()).collect();
        referred_tx_blocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let matching = referred_tx_blocks.iter().zip(expected.iter()).filter(|&(a, b)| a == b).count();
        assert_eq!(matching, expected.len());


        println!("Step 15:  Checking get_referred_prop_blocks() ");
        let mut referred_prop_blocks: Vec<H256> = blockchain.get_referred_prop_blocks(prop_block3.hash());
        let mut expected: Vec<H256> = vec![prop_block2b.hash()];
        referred_prop_blocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let matching = referred_prop_blocks.iter().zip(expected.iter()).filter(|&(a, b)| a == b).count();
        assert_eq!( matching, expected.len());

        println!("Step 16:  Checking get_votes_from_chain()");
        for i in 0..7 {
            let chain_votes = blockchain.get_votes_from_chain(i);
            let expected = vec![prop_block1.hash(), prop_block2a.hash(), prop_block3.hash(), prop_block4.hash()];
            let matching = chain_votes.iter().zip(expected.iter()).filter(|&(a, b)| a == b).count();
            assert_eq!( matching, expected.len());
        }
        for i in 7..10 {
            let chain_votes = blockchain.get_votes_from_chain(i);
            let expected = vec![prop_block1.hash(), prop_block2b.hash(), prop_block3.hash(), prop_block4.hash()];
            let matching = chain_votes.iter().zip(expected.iter()).filter(|&(a, b)| a == b).count();
            assert_eq!( matching, expected.len());
        }

        println!("Step 17:  Checking leader blocks for first four levels");
        let leader_block_sequence = blockchain.get_leader_block_sequence();
        assert_eq!(prop_block1.hash(), leader_block_sequence[0].unwrap());
        assert_eq!(prop_block2a.hash(),leader_block_sequence[1].unwrap());
        assert_eq!(prop_block3.hash(), leader_block_sequence[2].unwrap());
        assert_eq!(prop_block4.hash(), leader_block_sequence[3].unwrap());
        println!("\n");
    }

    // Debugging fn
    fn print_edges(blockchain: BlockChain) -> BlockChain {
        let all_edges = blockchain.graph.all_edges();
        for edge in all_edges{
           println!("Edge weight,{}", edge.2);
        }
        return blockchain;
    }
}