mod transaction;
mod proposer;
mod voter;
mod test_util;
use super::block::{Block, Content};
use super::crypto::hash::{Hashable, H256};
use serde::{Serialize, Deserialize};
use proposer::{ProposerNodeData, ProposerTree};
use voter::{VoterNodeData, VoterChain};
use std::collections::HashMap;

use petgraph::{Directed, Undirected, graph::NodeIndex};
use petgraph::graphmap::GraphMap;

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

// Make it cleaner?
impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        match self {
            Edge::TransactionToProposerParent => {write!(f,"Tx2PropParent"); Ok(())},
            Edge::ProposerToProposerParent => {write!(f,"Prop2PropParent"); Ok(())},
            Edge::ProposerToProposerReference => {write!(f,"Prop2PropRef"); Ok(())},
            Edge::ProposerToTransactionReference => {write!(f,"Prop2TxRef"); Ok(())},
            Edge::ProposerToTransactionLeaderReference => {write!(f,"Prop2TxLeaderRef"); Ok(())},
            Edge::VoterToProposerParent => {write!(f,"V2PropParent"); Ok(())},
            Edge::VoterToVoterParent => {write!(f,"V2VParent"); Ok(())},
            Edge::VoterToProposerVote => {write!(f,"V2PropVote"); Ok(())},
        }

    }
}

pub struct BlockChain{
    /// Store the three graph structures of Prism
    pub graph: GraphMap<H256, Edge, Undirected>,
    pub proposer_tree: ProposerTree,
    pub voter_chains: Vec<VoterChain>,
    /// Contains data about the proposer nodes.
    proposer_node_data_map: HashMap<H256, ProposerNodeData>,
    /// Contains data about the voter nodes.
    voter_node_data_map: HashMap<H256, VoterNodeData>
}

impl BlockChain {
    /// Used when the blockchain starts
    pub fn new(number_of_voter_chains: u16) -> Self {
        /// Initializing an empty objects
        let mut graph = GraphMap::<H256, Edge, Undirected>::new();
        let mut proposer_tree = ProposerTree::default();
        let mut voter_chains: Vec<VoterChain> = vec![];
        let mut proposer_node_data = HashMap::<H256, ProposerNodeData>::new();
        let mut voter_node_data = HashMap::<H256, VoterNodeData>::new();

        /// 1. Proposer genesis block
        /// 1a. Add proposer genesis block in the graph
        let proposer_genesis_node = ProposerNodeData::genesis(number_of_voter_chains);
        let proposer_hash_vec: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]; /// Hash vector of proposer genesis block. todo: Shift to a global config  file
        graph.add_node((&proposer_hash_vec).into());
        /// Add node data of proposer genesis block in the hashmap
        proposer_node_data.insert((&proposer_hash_vec).into(), proposer_genesis_node);
        // 1b. Initializing proposer tree
        proposer_tree.best_block= (&proposer_hash_vec).into();
        proposer_tree.prop_nodes.push(vec![(&proposer_hash_vec).into()]);
        proposer_tree.leader_nodes.push(Some((&proposer_hash_vec).into()));

        /// 2. Voter geneses blocks
        for chain_number in 0..(number_of_voter_chains) {
            /// 2a. Add voter chain i genesis block in the graph
            let voter_genesis_node = VoterNodeData::genesis(chain_number as u16);
            let b1 = ((chain_number+1) >> 8) as u8;
            let b2 = (chain_number+1) as u8;
            let voter_hash_vec: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,b1,b2]; /// Hash vector of voter genesis block. todo: Shift to a global config  file
            let voter_hash : H256 =  (&voter_hash_vec).into();
            graph.add_node(voter_hash);
            /// Add node data in the hashmap
            voter_node_data.insert(voter_hash, voter_genesis_node);
            /// 2b. Initializing a Voter chain
            let voter_chain = VoterChain::new(chain_number, voter_hash);
            voter_chains.push(voter_chain);
            proposer_tree.add_vote_at_level(voter_hash, 0);
        }
        return Self{graph, proposer_tree, voter_chains, proposer_node_data_map: proposer_node_data, voter_node_data_map: voter_node_data };
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
                /// Add edge from tx block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::TransactionToProposerParent);
            },

            Content::Proposer(content) => {
                /// 1, Add edge from prop block to its proposer parent
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::ProposerToProposerParent);

                /// 2. Iterate through the list of proposer blocks referred in the content of the given proposer block
                for prop_hash in content.proposer_block_hashes.iter(){
                    self.graph.add_edge(block_hash, *prop_hash, Edge::ProposerToProposerReference);
                }
                        println!("Number of tx blocks referred {}", content.transaction_block_hashes.len());

                /// 3. Iterate through the list of transaction block hashes referred in the content of the given proposer block
                for tx_hash in content.transaction_block_hashes.iter(){
                    self.graph.add_edge(block_hash, *tx_hash, Edge::ProposerToTransactionReference);
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
                self.graph.add_edge(block_hash, parent_proposer_block_hash, Edge::VoterToProposerParent);

                /// 2. Add edge from voter block to its voter parent
                self.graph.add_edge(block_hash, content.voter_parent_hash, Edge::VoterToVoterParent);

                for prop_block_hash in content.proposer_block_votes.iter() {
                    /// 3. Add edge from voter block to proposer votees
                    self.graph.add_edge(block_hash, (*prop_block_hash).clone(), Edge::VoterToProposerVote); // todo: (Caution) This removes the earlier proposer parent edge if it is present

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

    /// Get the best blocks on each voter chain
    pub fn get_voter_parents(&self) -> Vec<H256> {
        let voter_parents: Vec<H256> = self.voter_chains.iter().map(|&x| x.best_block).collect();
        return voter_parents;
    }

    /// Get the best block on proposer tree
    pub fn get_proposer_parent(&self) -> H256 {
        return self.proposer_tree.best_block;
    }

    /*
    Functions to add
    1. Longest chain on voter chain i
    2. Votes on the longest chain.
    3. Votes on a proposer level
    */

}


#[cfg(test)]
mod tests {
    use crate::crypto::hash::{H256};
    use super::*;
    use crate::block::generator as block_generator;
    use crate::block::{Block};
    use rand::{Rng, RngCore};
    use super::test_util;
    use petgraph_graphml::GraphMl;
    use std::fs;

    // At initialization the blockchain only consists of (m+1) genesis blocks.
    // The hash of these genesis nodes in the blockchain graph are fixed for now
    // because we have designed the genesis blocks themselves.
    #[test]
    fn blockchain_initialization(){
        /// Initialize a blockchain with 10  voter chains.
        let blockchain = BlockChain::new(10);

        /// Checking proposer tree's genesis block hash
        let proposer_genesis_hash_shouldbe: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]; /// Hash vector of proposer genesis block. todo: Shift to a global config  file
        let proposer_genesis_hash_shouldbe: H256 = (&proposer_genesis_hash_shouldbe).into();
        assert_eq!(proposer_genesis_hash_shouldbe, blockchain.proposer_tree.best_block);

        /// Checking all voter tree's genesis block hashes
        for chain_number in 0..10{
            let b1 = ((chain_number+1) >> 8) as u8;
            let b2 = (chain_number+1) as u8;
            let voter_genesis_hash_shouldbe: [u8; 32]   = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,b1,b2];/// Hash vector of voter genesis block. todo: Shift to a global config  file
            let voter_genesis_hash_shouldbe: H256 = (&voter_genesis_hash_shouldbe).into();
            assert_eq!(voter_genesis_hash_shouldbe, blockchain.voter_chains[chain_number as usize].best_block);
        }
    }

    #[test]
    fn blockchain_growing(){
        let mut rng = rand::thread_rng();
        let  n_voter_chains = 10;
        /// Initialize a blockchain with 10 voter chains.
        let mut blockchain = BlockChain::new(n_voter_chains);

        /// Store the parent blocks to mine on voter trees.
        let mut voter_parent_blocks: Vec<H256> = (0..n_voter_chains).map( |i| blockchain.voter_chains[i as usize].best_block).collect();// Currently the voter genesis blocks.

        /// Maintains the list of blocks of each type.
        let mut tx_block_vec: Vec<Block> = vec![];
        let mut unreferred_tx_block_index = 0;
        let mut prop_block_vec: Vec<Block> = vec![];
        let mut voter_block_vec: Vec<Vec<Block>> = vec![];


        println!("\nStep 1:  Initialized blockchain");
        assert_eq!(11, blockchain.graph.node_count(), "Expecting 11 nodes corresponding to 11 genesis blocks");
        assert_eq!(0, blockchain.graph.edge_count(), "Expecting 0 edges");
        println!("Result 1: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());



        println!("\nStep 2:   Added 5 tx blocks on prop genesis");
        /// Mine 5 tx block's with prop_best_block as the parent
        let tx_block_5: Vec<Block> = test_util::tx_blocks_with_parent_hash(5, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_5.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..5{ blockchain.add_block_as_node(tx_block_vec[i].clone()); }
        assert_eq!(16, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks");
        assert_eq!(5, blockchain.graph.edge_count(), "Expecting 5 edges");
        println!("Result 2: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());


        println!("\nStep 3:   Added prop block referring these 5 tx blocks");
        /// Generate a proposer block with prop_parent_block as the parent which referencing the above 5 tx blocks
        let prop_block1 = test_util::prop_block(blockchain.proposer_tree.best_block,
   tx_block_vec[0..5].iter().map( |x| x.hash()).collect(), vec![]);
        let prop_block1_hash: H256 = prop_block1.hash();
        /// Add the prop_block
        blockchain.add_block_as_node(prop_block1);
        assert_eq!(prop_block1_hash, blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(17, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(11, blockchain.graph.edge_count(), "Expecting 11 edges");
        println!("Result 3: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());


        println!("\nStep 4:    Add 10 voter blocks voting on proposer block at level 1");
        for i in 0..n_voter_chains{
            let voter_block = test_util::voter_block(blockchain.proposer_tree.best_block,
        i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block1_hash] );
            blockchain.add_block_as_node(voter_block);
        }
        assert_eq!(27, blockchain.graph.node_count());
        let prop_block1_votes =  blockchain.proposer_node_data_map[&prop_block1_hash].votes;
        assert_eq!(31, blockchain.graph.edge_count());
        assert_eq!(10, prop_block1_votes, "prop block 1 should have 10 votes" );
        println!("Result 4: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());
//        blockchain = print_edges(blockchain);


        println!("\nStep 5:   Mining 5 tx blocks, 2 prop blocks at level 2 with 3, 5 tx refs");
        unreferred_tx_block_index += 5;
        let tx_block_5: Vec<Block> = test_util::tx_blocks_with_parent_hash(5, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_5.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..5{ blockchain.add_block_as_node(tx_block_vec[unreferred_tx_block_index+i].clone()); }
        let prop_block2a = test_util::prop_block(blockchain.proposer_tree.best_block,
        tx_block_vec[5..8].iter().map( |x| x.hash()).collect(), vec![]); // Referring 3 tx blocks
        let prop_block2a_hash: H256 = prop_block2a.hash();
        blockchain.add_block_as_node(prop_block2a);
        assert_eq!(prop_block2a_hash, blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(33, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(40, blockchain.graph.edge_count(), "Expecting 11 edges");

        let prop_block2b = test_util::prop_block(prop_block1_hash,
        tx_block_vec[5..10].iter().map( |x| x.hash()).collect(), vec![]);// Referring 5 tx blocks
        let prop_block2b_hash: H256 = prop_block2b.hash();
        blockchain.add_block_as_node(prop_block2b);
        assert_ne!(prop_block2b_hash, blockchain.proposer_tree.best_block, "prop 2b is not best block");
        assert_eq!(34, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(46, blockchain.graph.edge_count(), "Expecting 11 edges");
        println!("Result 5: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 6:   Add 7+3 votes on proposer blocks at level 2");
        for i in 0..7{
            let voter_block = test_util::voter_block(prop_block2a_hash,
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block2a_hash] );
            blockchain.add_block_as_node(voter_block);
        }
        for i in 7..10{
            let voter_block = test_util::voter_block(prop_block2b_hash,
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block2b_hash] );
            blockchain.add_block_as_node(voter_block);
        }
        let prop_block2a_votes =  blockchain.proposer_node_data_map[&prop_block2a_hash].votes;
        let prop_block2b_votes =  blockchain.proposer_node_data_map[&prop_block2b_hash].votes;
        assert_eq!(7, prop_block2a_votes, "prop block 2a should have 7 votes" );
        assert_eq!(3, prop_block2b_votes, "prop block 2b should have 3 votes" );
        assert_eq!(10, blockchain.proposer_tree.all_votes[1].len(), "Level 2 total votes should have 10",);
        assert_eq!(44, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(66, blockchain.graph.edge_count(), "Expecting 11 edges");
        println!("Result 6: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 7:   Mining 4 tx block and 1 prop block referring 4 tx blocks + prop_block_2b)");
        unreferred_tx_block_index += 5;
        let tx_block_4: Vec<Block> = test_util::tx_blocks_with_parent_hash(4, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_4.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..4{ blockchain.add_block_as_node(tx_block_vec[unreferred_tx_block_index+i].clone()); }
        let prop_block3 = test_util::prop_block(blockchain.proposer_tree.best_block,
        tx_block_vec[10..14].iter().map( |x| x.hash()).collect(), vec![prop_block2b_hash]); // Referring 4 tx blocks + 1 prop_block
        let prop_block3_hash: H256 = prop_block3.hash();
        blockchain.add_block_as_node(prop_block3);
        assert_eq!(prop_block3_hash, blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(49, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(76, blockchain.graph.edge_count(), "Expecting 11 edges");
        println!("Result 7: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());


        println!("\nStep 8:  Mining only 3+3 voter blocks voting on none + prob_block3");
        for i in 0..3{
            let voter_block = test_util::voter_block(prop_block2a_hash, // Mining on 2a (because 3 hasnt showed up yet)
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![] );
            blockchain.add_block_as_node(voter_block);
        }
        for i in 3..6{
            let voter_block = test_util::voter_block(prop_block3_hash, // Mining on 3 after it showed up
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block3_hash] );
            blockchain.add_block_as_node(voter_block);
        }
        assert_eq!(55, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(88, blockchain.graph.edge_count(), "Expecting 11 edges");
        println!("Result 8: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 9:   Mining 2 tx block and 1 prop block referring the 2 tx blocks");
        unreferred_tx_block_index += 4;
        let tx_block_2: Vec<Block> = test_util::tx_blocks_with_parent_hash(2, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_2.iter().cloned());
        /// Add the tx blocks to blockchain
        for i in 0..2{ blockchain.add_block_as_node(tx_block_vec[unreferred_tx_block_index+i].clone()); }
        let prop_block4 = test_util::prop_block(blockchain.proposer_tree.best_block,
        tx_block_vec[14..16].iter().map( |x| x.hash()).collect(), vec![]); // Referring 4 tx blocks + 1 prop_block
        let prop_block4_hash: H256 = prop_block4.hash();
        blockchain.add_block_as_node(prop_block4);
        assert_eq!(prop_block4_hash, blockchain.proposer_tree.best_block, "Proposer best block");
        assert_eq!(58, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(93, blockchain.graph.edge_count(), "Expecting 11 edges");
        println!("Result 9: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        println!("\nStep 10: 1-6 voter chains vote on prop4 and 6-10 voter blocks vote on prop3 and prop4" );
        for i in 0..6{
            let voter_block = test_util::voter_block(prop_block4_hash, // Mining on 2a (because 3 hasnt showed up yet)
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block4_hash] );
            blockchain.add_block_as_node(voter_block);
        }
        for i in 6..10{
            let voter_block = test_util::voter_block(prop_block4_hash, // Mining on 3 after it showed up
            i as u16, blockchain.voter_chains[i as usize].best_block, vec![prop_block3_hash, prop_block4_hash] );
            blockchain.add_block_as_node(voter_block);
        }
        assert_eq!(68, blockchain.graph.node_count(), "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks and 1 prop block");
        assert_eq!(117, blockchain.graph.edge_count(), "Expecting 11 edges");
        println!("Result 9: Node count:{}, Edge count {}",blockchain.graph.node_count(), blockchain.graph.edge_count());

        /// Checking the voter chain growth
        for i in 0..6{
            assert_eq!(4, blockchain.voter_chains[i as usize].best_level);
        }   for i in 6..10{
            assert_eq!(3, blockchain.voter_chains[i as usize].best_level);
        }

        

        println!("\n");
//        print_edges(blockchain);
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
