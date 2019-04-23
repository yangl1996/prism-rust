pub mod edge;
pub mod proposer;
pub mod transaction;
pub mod utils;
pub mod voter;
use super::block::{Block, Content};
use super::crypto::hash::{Hashable, H256};
use crate::config::*;
use edge::Edge;
use petgraph::graphmap::GraphMap;
use petgraph::Directed;
use proposer::NodeData as ProposerNodeData;
use proposer::Status as ProposerStatus;
use proposer::Tree as ProposerTree;
use std::collections::VecDeque;
use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;
use std::sync::mpsc::Sender;
use transaction::Pool as TxBlkPool;
use transaction::UpdateMessage as LedgerUpdateMessage;
use utils::*;
use voter::Chain as VoterChain;
use voter::Fork as VoterChainFork;
use voter::NodeData as VoterNodeData;
use voter::NodeUpdateStatus as VoterNodeUpdateStatus;

pub struct BlockChain {
    /// Graph structure of the blockchain/graph
    pub graph: GraphMap<H256, Edge, Directed>,
    /// Metadata of the proposer tree
    pub proposer_tree: ProposerTree,
    /// Metadata of the voter chains
    pub voter_chains: Vec<VoterChain>,
    /// Metadata of the transaction blocks
    pub tx_blocks: TxBlkPool,
    /// Metadata of individual proposer blocks
    pub proposer_node_data: HashMap<H256, ProposerNodeData>,
    /// Metadata of individual voter blocks
    pub voter_node_data: HashMap<H256, VoterNodeData>,
}

// Functions to edit the blockchain
impl BlockChain {
    /// Initializing blockchain graph with genesis blocks.
    pub fn new(
        num_voting_chains: u16,
        utxo_update: Sender<(LedgerUpdateMessage, Vec<H256>)>, //Sends updates to utxo state thread
    ) -> Self {
        // Initializing an empty object
        let mut graph = GraphMap::<H256, Edge, Directed>::new();
        let mut proposer_tree = ProposerTree::default();
        let mut voter_chains: Vec<VoterChain> = vec![];
        let tx_blocks: TxBlkPool = TxBlkPool::new(utxo_update);
        let mut proposer_node_data = HashMap::<H256, ProposerNodeData>::new();
        let mut voter_node_data = HashMap::<H256, VoterNodeData>::new();

        // 1. Proposer genesis block
        // 1a. Add proposer genesis block in the graph
        let proposer_genesis_node = ProposerNodeData::genesis(num_voting_chains);
        graph.add_node(*PROPOSER_GENESIS);
        // Add metadata of the proposer genesis block to the hashmap
        proposer_node_data.insert(*PROPOSER_GENESIS, proposer_genesis_node);

        // 1b. Initializing proposer tree
        proposer_tree.best_block = *PROPOSER_GENESIS;
        proposer_tree.prop_nodes.push(vec![*PROPOSER_GENESIS]);
        // Genesis proposer block is the leader block at level 0
        proposer_tree.leader_nodes.insert(0, *PROPOSER_GENESIS);

        // 2. Voter geneses blocks
        for chain_number in 0..(num_voting_chains) {
            // 2a. Add voter chain i genesis block in the graph
            let voter_genesis_node = VoterNodeData::genesis(chain_number as u16);
            let voter_genesis_hash = VOTER_GENESIS[chain_number as usize];
            graph.add_node(voter_genesis_hash);
            // Add voter genesis node data to the hashmap
            voter_node_data.insert(voter_genesis_hash, voter_genesis_node);

            // 2b. Initializing a Voter chain
            let voter_chain = VoterChain::new(voter_genesis_hash);
            voter_chains.push(voter_chain);
            proposer_tree.increment_vote_at_level(0);
        }
        return Self {
            graph,
            proposer_tree,
            voter_chains,
            tx_blocks,
            proposer_node_data,
            voter_node_data,
        };
    }

    // TODO: add a function to restore BlockChain from BlockDatabase

    // Add edges from `from` to `to` with type `edge_type`, and edge from `to` to `from` with the
    // corresponding reversed type.
    fn insert_edge(&mut self, from: H256, to: H256, edge_type: Edge) {
        self.graph.add_edge(from, to, edge_type);
        if edge_type == Edge::VoterToProposerParentAndVote || edge_type == Edge::VoterToProposerVote
        {
            self.graph.add_edge(to, from, edge_type.reverse_edge());
        }
    }

    // Add a new block to the blockgraph. Note that all blocks referred by the new block must
    // already exist in the blockgraph.
    pub fn insert_node(&mut self, block: &Block) {
        let block_hash = block.hash();
        let parent_proposer_block_hash = block.header.parent_hash;
        // Add the node corresponding to the block in the graph field
        self.graph.add_node(block_hash);

        // Parse the block content and add the edges according to Prism logic.
        let content: &Content = &block.content;
        match content {
            Content::Transaction(_) => {
                // add edge from tx block to its proposer parent
                self.insert_edge(
                    block_hash,
                    parent_proposer_block_hash,
                    Edge::TransactionToProposerParent,
                );

                // mark this new tx block as not in ledger and as unreferred
                self.tx_blocks.insert_not_in_ledger(block_hash);
                self.tx_blocks.insert_unreferred(block_hash);
            }

            Content::Proposer(content) => {
                // 1. Add edge from prop block to its proposer parent
                self.insert_edge(
                    block_hash,
                    parent_proposer_block_hash,
                    Edge::ProposerToProposerParent,
                );
                // Since the parent proposer block is now referred, remove it from the unreferred proposer pool
                self.proposer_tree
                    .remove_unreferred(&parent_proposer_block_hash);
                // Mark the new block we just got as unreferred
                self.proposer_tree.insert_unreferred(block_hash);

                // 2. Iterate through the list of proposer blocks referred by this new block
                for (position, prop_hash) in content.proposer_block_hashes.iter().enumerate() {
                    self.insert_edge(
                        block_hash,
                        *prop_hash,
                        Edge::ProposerToProposerReference((1 + position) as u32),
                    ); // The parent prop block has the first position
                    self.proposer_tree.remove_unreferred(prop_hash);
                }

                // 3. Iterate through the list of transaction blocks referred by this new block and add an edge
                for (position, tx_hash) in content.transaction_block_hashes.iter().enumerate() {
                    self.insert_edge(
                        block_hash,
                        *tx_hash,
                        Edge::ProposerToTransactionReference(position as u32),
                    );
                    // Since the tx proposer block is now referred, remove it from the unreferred tx pool
                    self.tx_blocks.remove_unreferred(tx_hash);
                }

                // 4. Add the proposer block to the list of unvoted blocks on all the voter chains.
                let self_level = self.proposer_node_data[&parent_proposer_block_hash].level + 1;
                for i in 0..self.voter_chains.len() {
                    self.voter_chains
                        .get_mut(i as usize)
                        .unwrap()
                        .insert_unvoted(self_level, block_hash);
                }

                // 5. Creating proposer node data.
                // TODO: replace default with new
                let mut proposer_node_data = ProposerNodeData::default();
                proposer_node_data.level = self_level;

                // 6. Add node data in the map
                self.proposer_node_data
                    .insert(block_hash, proposer_node_data);

                // 7. Add the block in the proposer_tree field at self_level.
                self.proposer_tree
                    .add_block_at_level(block_hash, self_level);
            }

            Content::Voter(content) => {
                // 1, Add edge from voter block to its proposer parent
                self.insert_edge(
                    block_hash,
                    parent_proposer_block_hash,
                    Edge::VoterToProposerParent,
                );

                // 2. Add edge from voter block to its voter parent
                self.insert_edge(
                    block_hash,
                    content.voter_parent_hash,
                    Edge::VoterToVoterParent,
                );

                // 3. Add edge from voter block to proposer block it voted.
                for prop_block_hash in content.proposer_block_votes.iter() {
                    // if the votee is the same as the parent
                    if *prop_block_hash == parent_proposer_block_hash {
                        self.insert_edge(
                            block_hash,
                            *prop_block_hash,
                            Edge::VoterToProposerParentAndVote,
                        );
                    } else {
                        self.insert_edge(block_hash, *prop_block_hash, Edge::VoterToProposerVote);
                    }

                    // Incrementing the votes of the proposer block being voted
                    let ref mut proposer_node_data =
                        self.proposer_node_data.get_mut(&prop_block_hash).unwrap();
                    proposer_node_data.votes += 1;
                    self.proposer_tree
                        .increment_vote_at_level(proposer_node_data.level);
                }

                // 4. Updating the voter chain.
                let parent_voter_node_data: VoterNodeData =
                    self.voter_node_data[&content.voter_parent_hash];
                // The update status is used to create the voter node data
                let voter_node_update = self.voter_chains
                    [parent_voter_node_data.chain_number as usize]
                    .add_voter_block(
                        block_hash,
                        content.voter_parent_hash,
                        parent_voter_node_data.level + 1,
                    );

                // 5. Create voter node data.
                // Also update the unvoted_levels on the voter chain. For mining.
                // TODO: replace default with new
                let mut voter_node_data = VoterNodeData::default();
                voter_node_data.level = parent_voter_node_data.level + 1;
                voter_node_data.chain_number = parent_voter_node_data.chain_number;
                self.voter_node_data.insert(block_hash, voter_node_data);

                match voter_node_update {
                    // Case: New block extends the main chain. Good situation.
                    VoterNodeUpdateStatus::ExtendedMainChain => {
                        // change the status of the new voter block
                        self.voter_node_data
                            .get_mut(&block_hash)
                            .unwrap()
                            .make_on_main_chain();

                        // Remove the prop block levels voted by this block from list of unvoted levels of this chain.
                        for prop_block_hash in content.proposer_block_votes.iter() {
                            let proposer_level = self.prop_node_data(&prop_block_hash).level;
                            self.voter_chains
                                .get_mut(voter_node_data.chain_number as usize)
                                .unwrap()
                                .remove_unvoted(proposer_level);
                        }
                    }
                    // Case: New block is part of a side fork which is longer fork than the main chain.
                    // This is a bad (and complex) situation.
                    VoterNodeUpdateStatus::LongerFork => {
                        //1. Construct the Fork  object
                        let chain_number = voter_node_data.chain_number as usize;
                        let left_leaf = self.voter_chains[chain_number].best_block;
                        let left_leaf_level = self.voter_chains[chain_number].best_level;
                        let right_leaf = block_hash;
                        let right_leaf_level = voter_node_data.level;
                        let fork =
                            self.get_fork(left_leaf, right_leaf, left_leaf_level, right_leaf_level);
                        //2. Switch the main chain to the right segment of the fork
                        let new_best_block = fork.right_segment.last().unwrap();
                        let new_best_level = self.voter_node_data(new_best_block).level;
                        self.voter_chains[chain_number]
                            .switch_the_main_chain(*new_best_block, new_best_level);

                        //3a. Change the status of left_segment voter blocks to Orphan and get the votes by this segment.
                        let mut votes_on_proposers_left: Vec<(H256, u32)> = vec![];
                        for voter_block in fork.left_segment.iter() {
                            self.voter_node_data
                                .get_mut(voter_block)
                                .unwrap()
                                .make_orphan(); //change status
                            let proposer_blocks = self.get_votes_by_voter(voter_block);
                            for proposer_block in proposer_blocks {
                                let level = self.prop_node_data(&proposer_block).level;
                                votes_on_proposers_left.push((proposer_block, level));
                            }
                            votes_on_proposers_left.sort_by_key(|k| k.1);
                        }

                        let first_left_segment_vote_level = votes_on_proposers_left[0].1; // This will be used for rollback

                        //3b. Change the status of right_segment voter blocks to OnMainChain and get the votes by this segment.
                        let mut votes_on_proposers_right: Vec<(H256, u32)> = vec![];
                        for voter_block in fork.right_segment.iter() {
                            self.voter_node_data
                                .get_mut(voter_block)
                                .unwrap()
                                .make_on_main_chain(); //change status
                            let proposer_blocks = self.get_votes_by_voter(voter_block);
                            for proposer_block in proposer_blocks {
                                let level = self.prop_node_data(&proposer_block).level;
                                votes_on_proposers_right.push((proposer_block, level));
                            }
                            votes_on_proposers_right.sort_by_key(|k| k.1);
                        }

                        //4a. Add votes_on_proposers_left to unvoted_proposer_blocks in voter chain. For mining.
                        self.voter_chains[chain_number]
                            .add_unvoted_while_switching(votes_on_proposers_left);

                        //4b. Remove the votes_on_proposers_left to unvoted_proposer_blocks in voter chain. For mining.
                        self.voter_chains[chain_number]
                            .remove_unvoted_while_switching(votes_on_proposers_right);

                        //5 Rollback the ledger if required.

                        //5a. Check if the leader blocks have changed between first_left_segment_vote_level and min_unconfirmed_level.
                        let mut roll_back_level = 0;
                        let mut roll_back_required = false;
                        for level in
                            first_left_segment_vote_level..self.proposer_tree.min_unconfirmed_level
                        {
                            let old_leader_block = self.get_leader_block_at_level(level).unwrap();
                            match self.compute_leader_block_at_level(level) {
                                Some(new_leader_block) => {
                                    if old_leader_block != new_leader_block {
                                        // Leader block changed at the level
                                        roll_back_required = true;
                                        roll_back_level = level;
                                        break;
                                    }
                                }
                                None => {
                                    // level leader block is not the leader block and infact level has not leader block
                                    self.proposer_tree.leader_nodes.remove(&level);
                                    roll_back_required = true;
                                    roll_back_level = level;
                                    break;
                                }
                            }
                        }

                        if roll_back_required {
                            println!("51% attack, roll back at level {}", roll_back_level);
                            //5b. Change status of all the proposer blocks from level roll_back_level onwards to Potential Leader
                            for level in roll_back_level..self.proposer_tree.min_unconfirmed_level {
                                for proposer_block in
                                    self.proposer_tree.prop_nodes[level as usize].iter()
                                {
                                    self.proposer_node_data
                                        .get_mut(proposer_block)
                                        .unwrap()
                                        .give_potential_leader_status();
                                }

                                self.proposer_tree.leader_nodes.remove(&level);
                            }
                            //5c. Rollback ledger from 'roll_back_level' level onwards.
                            self.tx_blocks.rollback_ledger(roll_back_level as usize);
                            self.proposer_tree.min_unconfirmed_level = roll_back_level as u32;
                        }
                    }
                    // Case: A orphan voter block. Do nothing.
                    VoterNodeUpdateStatus::SideChain => {}
                }
                // Try confirming levels from the min unconfirmed proposer level.
                loop {
                    let level = self.proposer_tree.min_unconfirmed_level;
                    self.try_confirm_leader_block_at_level(level);
                    // Exit the loop if previous step did not increase "self.proposer_tree.min_unconfirmed_level"
                    if level == self.proposer_tree.min_unconfirmed_level {
                        break;
                    }
                }
            }
        };
    }

    fn get_fork(
        &self,
        left_leaf: H256,
        right_leaf: H256,
        left_leaf_level: u32,
        right_leaf_level: u32,
    ) -> VoterChainFork {
        if left_leaf_level >= right_leaf_level {
            panic!("This function should not be called when a small fork appears")
        }
        let mut left_segment: Vec<H256> = vec![];
        let mut right_segment: Vec<H256> = vec![];
        let mut left_end: H256 = left_leaf;
        let mut right_end: H256 = right_leaf;

        //1. Construct right segment until the level of right_end is same as left_end
        for level in left_leaf_level..right_leaf_level {
            right_segment.push(right_end);
            right_end = self.get_voter_parent(right_end);
        }

        loop {
            // If the ends are the same, then we've found a common parent
            if left_end == right_end {
                left_segment.reverse();
                right_segment.reverse();
                return VoterChainFork {
                    common_parent: left_end,
                    left_segment,
                    right_segment,
                };
            }
            // Extends both the segments.
            right_segment.push(right_end);
            right_end = self.get_voter_parent(right_end);

            left_segment.push(left_end);
            left_end = self.get_voter_parent(left_end);
        }
    }
}

// Functions to infer the voter chains. These functions are not currently used in logic but they are tested.
impl BlockChain {
    /// Return the voter blocks on the longest voter chain `chain_number`
    pub fn get_longest_chain(&self, chain_number: u16) -> Vec<H256> {
        let chain = &self.voter_chains[chain_number as usize];
        let best_level = chain.best_level;
        let mut longest_chain: Vec<H256> = vec![];
        let mut top_block: H256 = chain.best_block;

        // Recursively push the top block
        // TODO: I have a sense that this can be realized using Option and collect
        for _ in 0..best_level {
            longest_chain.push(top_block);
            top_block = self.get_voter_parent(top_block);
        }
        longest_chain.push(top_block);
        longest_chain.reverse();
        return longest_chain;
    }

    /// Return the hashes of the proposer blocks voted by a voter chain.
    pub fn get_votes_from_chain(&self, chain_number: u16) -> Vec<H256> {
        let longest_chain: Vec<H256> = self.get_longest_chain(chain_number);
        let mut votes: Vec<H256> = vec![];
        for voter in longest_chain {
            let mut voter_votes = self.get_votes_by_voter(&voter);
            voter_votes.reverse(); // TODO: Why? Ordering?
            votes.extend(voter_votes);
        }
        return votes;
    }

    /// Return the hashes of the proposer blocks voted by a single voter block.
    pub fn get_votes_by_voter(&self, block_hash: &H256) -> Vec<H256> {
        if !self.voter_node_data.contains_key(&block_hash) {
            panic!("The voter block with hash {} doesn't exist", block_hash);
        }
        let voter_ref_edges = self.graph.edges(*block_hash).filter(|&x| {
            *x.2 == Edge::VoterToProposerVote || *x.2 == Edge::VoterToProposerParentAndVote
        });
        let voter_ref_nodes: Vec<H256> = voter_ref_edges.map(|x| x.1).collect();
        return voter_ref_nodes;
    }

    /// Return the voter parent of a voter block
    pub fn get_voter_parent(&self, block_hash: H256) -> H256 {
        if !self.voter_node_data.contains_key(&block_hash) {
            panic!("The voter block with hash {} doesn't exist", block_hash);
        }
        let voter_parent_edges = self
            .graph
            .edges(block_hash)
            .filter(|&x| *x.2 == Edge::VoterToVoterParent);
        let voter_parent_nodes: Vec<H256> = voter_parent_edges.map(|x| x.1).collect();
        if voter_parent_nodes.len() == 1 {
            return voter_parent_nodes[0];
        } else {
            panic!(
                "{} proposer parents for {}",
                voter_parent_nodes.len(),
                block_hash
            )
        }
    }

    pub fn number_of_voting_chains(&self) -> u32 {
        return self.voter_chains.len() as u32;
    }
}

// Functions to generate the ledger. This uses the confirmation logic of Prism.
impl BlockChain {
    /// Returns the list of ordered tx blocks. This is the initial step of creating the full ledger.
    pub fn get_ordered_tx_blocks(&self) -> &Vec<H256> {
        return &self.tx_blocks.ledger;
    }

    /// Checks if there are sufficient votes to confirm leader block at the level.
    /// If yes it confirms the leader block at the level and updates the ledger. Else it does nothing.
    // TODO: This function should be called when the voter chain has collected sufficient votes on level.
    pub fn try_confirm_leader_block_at_level(&mut self, level: u32) {
        if self.proposer_tree.leader_nodes.contains_key(&level) {
            return; // Return if the level already has a confirmed leader block.
        }

        if self.proposer_tree.prop_nodes.len() <= level as usize {
            return; // Return if the level has no proposer blocks.
        }

        if !self.proposer_tree.number_of_votes.contains_key(&level) {
            return; // Return if no votes are cast on the level
        }
        if self.proposer_tree.number_of_votes[&level] < self.number_of_voting_chains() / 2 {
            return; // Return if less than half the votes are be caste. This is only for efficiency
        }

        let mut leader_block;
        match self.compute_leader_block_at_level(level) {
            Some(x) => leader_block = x,
            None => return,
        }

        // 2a. Adding the leader block for the level
        self.proposer_tree.leader_nodes.insert(level, leader_block);
        self.proposer_tree.min_unconfirmed_level = level + 1;

        // 2b. Giving leader status to leader_block
        let ref mut leader_node_data = self.proposer_node_data.get_mut(&leader_block).unwrap();
        leader_node_data.give_leader_status();

        // 2c. Giving NotLeaderUnconfirmed status to all blocks at 'level' except the leader_block
        for proposer_block in self.proposer_tree.prop_nodes[level as usize].iter() {
            if *proposer_block != leader_block {
                let ref mut proposer_node_data =
                    self.proposer_node_data.get_mut(proposer_block).unwrap();
                proposer_node_data.give_not_leader_status();
            }
        }

        // 3. Updating ledger because a new leader block is added.
        self.update_ledger(level);
    }

    /// Computes the leader block at the level using the  voter chains
    pub fn compute_leader_block_at_level(&self, level: u32) -> Option<H256> {
        //0. Get the list of proposer blocks at the level.
        let proposers_blocks: &Vec<H256> = &self.proposer_tree.prop_nodes[level as usize];

        // 1. Getting the lcb of votes on each proposer block and  the block with max_lcb votes.
        let mut lcb_proposer_votes: HashMap<H256, f32> = HashMap::<H256, f32>::new();
        let mut max_lcb_vote: f32 = -1.0;
        let mut max_lcb_vote_index: usize = 0;
        // Stores the number votes which have not been caste (or are still not permanent).
        let mut left_over_votes: f32 = self.voter_chains.len() as f32;

        // todo: This seems inefficient. Also equal votes situation is not considered.
        for (index, proposer) in proposers_blocks.iter().enumerate() {
            let proposer_votes: Vec<u32> = self.get_vote_depths_on_proposer(proposer);
            let lcb = utils::lcb_from_vote_depths(proposer_votes);
            lcb_proposer_votes.insert(*proposer, lcb);
            left_over_votes -= lcb; // removing the cast (or permanent) votes
            if max_lcb_vote < lcb {
                max_lcb_vote = lcb;
                max_lcb_vote_index = index;
            }
        }
        // It the left over votes is more than the votes received by the max_lcb_vote_index block, then
        // dont confirm because a private proposer block could potentially get all these left over votes.
        // and become the leader block of that level.
        if left_over_votes >= max_lcb_vote {
            return None;
        }
        // TODO: The fast confirmation can be done here
        // Dont confirm if another proposer block could potentially get all these left over votes.
        for (index, proposer) in proposers_blocks.iter().enumerate() {
            let ucb = lcb_proposer_votes[proposer] + left_over_votes;
            if index == max_lcb_vote_index {
                continue;
            }
            if ucb >= max_lcb_vote {
                return None;
            }
        }

        // If the function reaches here, the 'level' has a proposer block with maximum votes. Yay.
        return Some(proposers_blocks[max_lcb_vote_index]);
    }

    /// For all the votes (on the voter chain) for a given proposer block, return the depth of
    /// those votes, where depth is the number of children voter blocks on the vote.
    pub fn get_vote_depths_on_proposer(&self, block_hash: &H256) -> Vec<u32> {
        if !self.proposer_node_data.contains_key(block_hash) {
            panic!("The proposer block with hash {} doesn't exist", block_hash);
        }
        //1. Extracting the voter blocks which have voted on this proposer block
        let voter_edges = self.graph.edges(*block_hash).filter(|&x| {
            *x.2 == Edge::VoterFromProposerVote || *x.2 == Edge::VoterFromProposerParentAndVote
        });
        let mut voter_depths: Vec<u32> = vec![];
        for edge in voter_edges {
            //2a. Filter out votes which come from non-main-chain voter blocks
            let voter_block_hash = edge.1;
            let voter_node_data = self.voter_node_data[&voter_block_hash];
            if !voter_node_data.is_on_main_chain() {
                continue;
            }
            //2b Get the depth of the voter
            let voter_level = voter_node_data.level;
            let voter_chain_number = voter_node_data.chain_number;
            let voter_chain_depth = self.voter_chains[voter_chain_number as usize].best_level;
            voter_depths.push(voter_chain_depth - voter_level + 1);
        }
        return voter_depths;
    }

    /// Called when a new leader block is confirmed at some level.
    fn update_ledger(&mut self, level: u32) {
        let leader_block = self.proposer_tree.leader_nodes[&level];

        //1. Get all the proposer blocks referred by this leader block which are not confirmed and
        // aren't themselves leader blocks on their level. We need an *ordered* list here.
        // Reason: We can confirm these proposer blocks and thereby confirming the tx blocks referred by these proposer blocks.
        let to_confirm_proposer_blocks =
            self.get_unconfirmed_notleader_referred_proposer_blocks(leader_block);

        //2. Add the transactions blocks referred by these proposer blocks to the ledger.
        self.tx_blocks.mark_confirmation_boundary(level);
        for proposer_block in to_confirm_proposer_blocks.iter() {
            // Get all the tx blocks referred.
            for tx_block in self.get_referred_tx_blocks_ordered(proposer_block) {
                // Add the tx block to the ledger if not already in the ledger
                let mut tx_blocks_to_add: Vec<H256> = vec![];
                if !self.tx_blocks.is_in_ledger(&tx_block) {
                    tx_blocks_to_add.push(tx_block);
                }
                self.tx_blocks.add_to_ledger(tx_blocks_to_add);
            }
            // Changing the status of these prop blocks to 'not leader but confirmed'.
            // Reason: This is done to prevent these prop blocks from getting confirmed again.
            if *proposer_block != leader_block {
                self.proposer_node_data
                    .get_mut(proposer_block)
                    .unwrap()
                    .give_not_leader_confirmed_status();
            }
        }
    }

    /// Idea: When a new leader block, B, is confirmed, it also confirms
    /// 'notleader and unconfirmed' proposer blocks directly and indirectly referred by B.
    /// Let set S be the list of all the 'notleader and unconfirmed' proposer blocks referred by B.
    /// The function obtains S and orders them according the the following rule
    /// Topologically ordering rule: for forall B1, B2 \in S:
    /// 1. If B1.level < B2.level ==> B1 < B2.
    /// 2. If B1.level == B2.level, then B1 < B2 if B1 is (directly or indirectly) referred before B2 in block B.
    fn get_unconfirmed_notleader_referred_proposer_blocks(&self, block_hash: H256) -> Vec<H256> {
        let mut all_blocks: BTreeMap<H256, PropOrderingHelper> = BTreeMap::new(); // Will store the set S.

        //Algo: Run DFS graph traversal to obtain and order the 'notleader and unconfirmed' prop blocks.
        // Each block has a 'position' and is ordered according to the position.

        // The queue is used for DFS traversal. The topological ordering logic is present in struct PropOrderingHelper.
        let mut queue: VecDeque<(H256, PropOrderingHelper)> = VecDeque::new();
        let node_data = self.prop_node_data(&block_hash);
        queue.push_back((
            block_hash,
            PropOrderingHelper::new(node_data.level, vec![0]),
        ));

        while let Some(block) = queue.pop_front() {
            let block_hash = block.0.clone(); // next block in the traversal path
            let new_position = block.1.clone();

            //1. Add the block to all_blocks.

            //If the block is already present in all_blocks, remove it if the new position is better than the previous.
            if all_blocks.contains_key(&block.0) {
                let old_position = &all_blocks[&block.0];
                if new_position < *old_position {
                    all_blocks.remove(&block.0);
                } else {
                    continue; // skip the loop to go the next block in the DFS traversal path
                }
            }
            // Add the block to all_blocks
            all_blocks.insert(block.0, block.1);

            //2. Add the block's parent and referred notleader proposer blocks to the DFS traversal queue.
            let referred_prop_blocks =
                self.get_unconfirmed_notleader_referred_proposer_blocks_prev_level(block_hash);
            for block in referred_prop_blocks {
                let block_h = block.0;
                let node_data = self.prop_node_data(&block_h);
                let level = node_data.level; // level in the prop tree
                let mut new_position_vec = new_position.position.clone();
                new_position_vec.push(block.1);
                queue.push_back((block_h, PropOrderingHelper::new(level, new_position_vec)));
            }
        }
        // Order all_blocks using via comparision logic in the comment of the function which is
        // coded in PropOrderingHelper.
        let mut v_all_blocks = Vec::from_iter(all_blocks);
        v_all_blocks.sort_by(|(_, a), (_, b)| a.cmp(&b));

        let answer: Vec<H256> = v_all_blocks.into_iter().map(|(x, _)| x).collect();
        return answer;
    }

    /// Returns the tx blocks directly referred by the proposer block
    pub fn get_referred_tx_blocks_ordered(&self, block_hash: &H256) -> Vec<H256> {
        if !self.proposer_node_data.contains_key(block_hash) {
            panic!("The proposer block with hash {} doesn't exist", *block_hash);
        }
        let all_edges = self.graph.edges(*block_hash);

        let mut referred_tx_blocks_nodes: Vec<(H256, u32)> = vec![];
        for edge in all_edges {
            if let Edge::ProposerToTransactionReference(position) = *edge.2 {
                referred_tx_blocks_nodes.push((edge.1, position));
            }
        }
        referred_tx_blocks_nodes.sort_by_key(|k| k.1);
        // returning the hashes only
        return referred_tx_blocks_nodes
            .into_iter()
            .map(|(x, _)| x)
            .collect();
    }

    /// Returns all the notleader proposer blocks references by the block_hash including the parent on the previous level of the  block hash.
    /// The blocks are ordered by their position in the reference list.
    fn get_unconfirmed_notleader_referred_proposer_blocks_prev_level(
        &self,
        block_hash: H256,
    ) -> Vec<(H256, u32)> {
        let parent_block: H256 = self.get_proposer_parent(block_hash);
        let mut referred_prop_blocks: Vec<(H256, u32)> = self.get_referred_prop_blocks(block_hash);
        referred_prop_blocks.push((parent_block, 0));

        // Filtering only NotLeader and Unconfirmed Blocks
        let mut filtered_referred_prop_blocks: Vec<(H256, u32)> = referred_prop_blocks
            .into_iter()
            .filter(|&x| {
                self.prop_node_data(&x.0).leadership_status == ProposerStatus::NotLeaderUnconfirmed
            })
            .collect();
        // Order the proposer blocks by their edge number
        filtered_referred_prop_blocks.sort_by_key(|k| k.1);

        return filtered_referred_prop_blocks;
    }

    /// Return the proposer parent of the block
    pub fn get_proposer_parent(&self, block_hash: H256) -> H256 {
        let proposer_parent_edges = self.graph.edges(block_hash).filter(|&x| {
            (*x.2 == Edge::TransactionToProposerParent
                || *x.2 == Edge::ProposerToProposerParent
                || *x.2 == Edge::VoterToProposerParent
                || *x.2 == Edge::VoterToProposerParentAndVote)
        });
        let proposer_parent_nodes: Vec<H256> = proposer_parent_edges.map(|x| x.1).collect();
        if proposer_parent_nodes.len() == 1 {
            return proposer_parent_nodes[0];
        } else {
            for edge in self.graph.edges(block_hash) {
                println!("Neighs {} with type {}", edge.1, edge.2);
            }
            panic!(
                "{} proposer parents for {}",
                proposer_parent_nodes.len(),
                block_hash
            )
        }
    }

    /// Returns the prop blocks directly referred by the proposer block
    pub fn get_referred_prop_blocks(&self, block_hash: H256) -> Vec<(H256, u32)> {
        if !self.proposer_node_data.contains_key(&block_hash) {
            panic!("The proposer block with hash {} doesn't exist", block_hash);
        }
        let all_edges = self.graph.edges(block_hash);

        let mut referred_prop_blocks_nodes: Vec<(H256, u32)> = vec![];
        for edge in all_edges {
            if let Edge::ProposerToProposerReference(position) = *edge.2 {
                referred_prop_blocks_nodes.push((edge.1, position));
            }
        }
        return referred_prop_blocks_nodes;
    }

    /// Return a single leader block at the given level
    fn get_leader_block_at_level(&self, level: u32) -> Option<H256> {
        if self.proposer_tree.leader_nodes.contains_key(&level) {
            return Some(self.proposer_tree.leader_nodes[&level]);
        }
        return None;
    }

    /// Returns the leader blocks from 0 to best level of the proposer tree
    pub fn get_leader_block_sequence(&mut self) -> Vec<H256> {
        let leader_blocks: Vec<H256> = (1..self.proposer_tree.min_unconfirmed_level)
            .map(|level| self.get_leader_block_at_level(level).unwrap())
            .collect();
        return leader_blocks;
    }
}

/// Functions for mining.
impl BlockChain {
    /// Return the best blocks on voter chain 'chain number'.
    pub fn get_voter_best_block(&self, chain_number: u16) -> H256 {
        let voter_best_block: H256 = self.voter_chains[chain_number as usize].best_block;
        return voter_best_block;
    }

    /// Return the best block on proposer tree.s
    pub fn get_proposer_best_block(&self) -> H256 {
        return self.proposer_tree.best_block;
    }

    /// Proposer block content 1
    pub fn get_unreferred_prop_blocks(&self) -> Vec<H256> {
        let mut unreferred_prop_blocks = self.proposer_tree.unreferred.clone();
        // Remove the parent block
        unreferred_prop_blocks.remove(&self.get_proposer_best_block());
        return Vec::from_iter(unreferred_prop_blocks);
    }

    /// Proposer block content 2
    pub fn get_unreferred_tx_blocks(&self) -> Vec<H256> {
        let unreferred_tx_blocks = self.tx_blocks.unreferred.clone();
        return Vec::from_iter(unreferred_tx_blocks);
    }

    /// Voter block content
    pub fn get_unvoted_prop_blocks(&self, chain_number: u16) -> Vec<H256> {
        return self.voter_chains[chain_number as usize].get_unvoted_prop_blocks();
    }

    //The content for transaction blocks is maintained in the tx-mempool, not here.
}

/// Functions for block validation
impl BlockChain {
    pub fn check_node(&self, hash: H256) -> bool {
        return self.graph.contains_node(hash);
    }
}

/// Helper functions: Get proposer and node data
impl BlockChain {
    pub fn prop_node_data(&self, hash: &H256) -> &ProposerNodeData {
        return &self.proposer_node_data[hash];
    }

    pub fn voter_node_data(&self, hash: &H256) -> &VoterNodeData {
        return &self.voter_node_data[hash];
    }
}

#[cfg(test)]
mod tests {
    use super::utils;
    use super::*;
    use crate::block::Block;
    use crate::crypto::hash::H256;
    const NUM_VOTER_CHAINS: u16 = 10;
    use std::sync::mpsc;

    // At initialization the blockchain only consists of (m+1) genesis blocks.
    // The hash of these genesis nodes in the blockchain graph are fixed for now
    // because we have designed the genesis blocks themselves.
    #[test]
    fn blockchain_initialization() {
        // Initialize a blockchain with 10  voter chains.
        let (state_update_sink, state_update_source) = mpsc::channel();

        let mut blockchain = BlockChain::new(NUM_VOTER_CHAINS, state_update_sink);

        // Checking proposer tree's genesis block hash
        let proposer_genesis_hash_shouldbe: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        // Hash vector of proposer genesis block. todo: Shift to a global config  file
        let proposer_genesis_hash_shouldbe: H256 = (&proposer_genesis_hash_shouldbe).into();
        assert_eq!(
            proposer_genesis_hash_shouldbe,
            blockchain.proposer_tree.best_block
        );

        // Checking all voter tree's genesis block hashes
        for chain_number in 0..NUM_VOTER_CHAINS {
            let b1 = ((chain_number + 1) >> 8) as u8;
            let b2 = (chain_number + 1) as u8;
            let voter_genesis_hash_shouldbe: [u8; 32] = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, b1, b2,
            ];
            // Hash vector of voter genesis block. todo: Shift to a global config  file
            let voter_genesis_hash_shouldbe: H256 = (&voter_genesis_hash_shouldbe).into();
            assert_eq!(
                voter_genesis_hash_shouldbe,
                blockchain.voter_chains[chain_number as usize].best_block
            );
        }
    }

    #[test]
    fn blockchain_growing() {
        let _rng = rand::thread_rng();
        // Initialize a blockchain with 10 voter chains.
        let (state_update_sink, state_update_source) = mpsc::channel();
        let mut blockchain = BlockChain::new(NUM_VOTER_CHAINS, state_update_sink);

        // Store the parent blocks to mine on voter trees.
        let mut voter_best_blocks: Vec<H256> = (0..NUM_VOTER_CHAINS)
            .map(|i| blockchain.voter_chains[i as usize].best_block)
            .collect(); // Currently the voter genesis blocks.

        let proposer_genesis = blockchain.proposer_tree.best_block;
        let voter_genesis_blocks = voter_best_blocks.clone();
        // Maintains the list of tx blocks.
        let mut tx_block_vec: Vec<Block> = vec![];
        let mut unreferred_tx_block_index = 0;

        println!("Test 1:   Initialized blockchain");
        assert_eq!(
            11,
            blockchain.graph.node_count(),
            "Expecting 11 nodes corresponding to 11 genesis blocks"
        );
        assert_eq!(0, blockchain.graph.edge_count(), "Expecting 0 edges");

        println!("Test 2:   Added 5 tx blocks on prop genesis");
        // Mine 5 tx block's with prop_best_block as the parent
        let tx_block_5: Vec<Block> =
            utils::test_tx_blocks_with_parent_hash(5, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_5.iter().cloned());
        // Add the tx blocks to blockchain
        for i in 0..5 {
            blockchain.insert_node(&tx_block_vec[i]);
        }
        assert_eq!(
            16,
            blockchain.graph.node_count(),
            "Expecting 16 nodes corresponding to 11 genesis blocks and  5 tx blocks"
        );
        assert_eq!(5, blockchain.graph.edge_count(), "Expecting 10 edges");

        println!("Test 3:   Added prop block referring these 5 tx blocks");
        // Generate a proposer block with prop_parent_block as the parent which referencing the above 5 tx blocks
        let prop_block1a = utils::test_prop_block(
            blockchain.proposer_tree.best_block,
            tx_block_vec[0..5].iter().map(|x| x.hash()).collect(),
            vec![],
        );
        // Add the prop_block
        blockchain.insert_node(&prop_block1a);
        assert_eq!(
            prop_block1a.hash(),
            blockchain.proposer_tree.best_block,
            "Proposer best block"
        );
        assert_eq!(17, blockchain.graph.node_count(), "Expecting 17 nodes");
        assert_eq!(11, blockchain.graph.edge_count(), "Expecting 22 edges");

        println!("Test 4:   Add 10 voter blocks voting on proposer block at level 1");
        for i in 0..NUM_VOTER_CHAINS {
            assert_eq!(
                1,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block1a.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            let voter_block = utils::test_voter_block(
                blockchain.proposer_tree.best_block,
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks(),
            );
            blockchain.insert_node(&voter_block);
        }
        assert_eq!(27, blockchain.graph.node_count());
        let prop_block1a_votes = blockchain.proposer_node_data[&prop_block1a.hash()].votes;
        assert_eq!(41, blockchain.graph.edge_count());
        assert_eq!(10, prop_block1a_votes, "prop block 1 should have 10 votes");

        println!("Test 5:   Mining 5 tx blocks, 2 prop blocks at level 2 with 3, 5 tx refs");
        unreferred_tx_block_index += 5;
        let tx_block_5: Vec<Block> =
            utils::test_tx_blocks_with_parent_hash(5, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_5.iter().cloned());
        // Add the tx blocks to blockchain
        for i in 0..5 {
            blockchain.insert_node(&tx_block_vec[unreferred_tx_block_index + i]);
        }
        let prop_block2a = utils::test_prop_block(
            blockchain.proposer_tree.best_block,
            tx_block_vec[5..8].iter().map(|x| x.hash()).collect(),
            vec![],
        ); // Referring 3 tx blocks
        blockchain.insert_node(&prop_block2a);
        assert_eq!(
            prop_block2a.hash(),
            blockchain.proposer_tree.best_block,
            "Proposer best block"
        );
        assert_eq!(33, blockchain.graph.node_count(), "Expecting 33 nodes");
        assert_eq!(50, blockchain.graph.edge_count(), "Expecting 80 edges");

        let prop_block2b = utils::test_prop_block(
            prop_block1a.hash(),
            tx_block_vec[5..10].iter().map(|x| x.hash()).collect(),
            vec![],
        ); // Referring 5 tx blocks
        blockchain.insert_node(&prop_block2b);
        assert_ne!(
            prop_block2b.hash(),
            blockchain.proposer_tree.best_block,
            "prop 2b is not best block"
        );
        assert_eq!(34, blockchain.graph.node_count(), "Expecting 34 nodes");
        assert_eq!(56, blockchain.graph.edge_count(), "Expecting 92 edges");

        println!("Test 6:   Add 7+3 votes on proposer blocks at level 2");
        for i in 0..7 {
            assert_eq!(
                1,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block2a.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            let voter_block = utils::test_voter_block(
                prop_block2a.hash(),
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks(),
            );
            blockchain.insert_node(&voter_block);
        }
        for i in 7..10 {
            assert_eq!(
                1,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block2a.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            // We are instead voting on prop block 2b
            let voter_block = utils::test_voter_block(
                prop_block2b.hash(),
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                vec![prop_block2b.hash()],
            );
            blockchain.insert_node(&voter_block);
        }
        let prop_block2a_votes = blockchain.proposer_node_data[&prop_block2a.hash()].votes;
        let prop_block2b_votes = blockchain.proposer_node_data[&prop_block2b.hash()].votes;
        assert_eq!(7, prop_block2a_votes, "prop block 2a should have 7 votes");
        assert_eq!(3, prop_block2b_votes, "prop block 2b should have 3 votes");
        assert_eq!(
            10, blockchain.proposer_tree.number_of_votes[&1],
            "Level 2 total votes should have 10",
        );
        assert_eq!(44, blockchain.graph.node_count(), "Expecting 44 nodes");
        assert_eq!(86, blockchain.graph.edge_count(), "Expecting 132 edges");

        println!(
            "Test 7:   Mining 4 tx block and 1 prop block referring 4 tx blocks + prop_block_2b)"
        );
        unreferred_tx_block_index += 5;
        let tx_block_4: Vec<Block> =
            utils::test_tx_blocks_with_parent_hash(4, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_4.iter().cloned());
        // Add the tx blocks to blockchain
        for i in 0..4 {
            blockchain.insert_node(&tx_block_vec[unreferred_tx_block_index + i]);
        }
        let prop_block3 = utils::test_prop_block(
            blockchain.proposer_tree.best_block,
            tx_block_vec[10..14].iter().map(|x| x.hash()).collect(),
            vec![prop_block2b.hash()],
        ); // Referring 4 tx blocks + 1 prop_block
        blockchain.insert_node(&prop_block3);
        assert_eq!(
            prop_block3.hash(),
            blockchain.proposer_tree.best_block,
            "Proposer best block"
        );
        assert_eq!(49, blockchain.graph.node_count(), "Expecting 49 nodes");
        assert_eq!(96, blockchain.graph.edge_count(), "Expecting 152 edges");

        println!("Test 8:   Mining only 3+3 voter blocks voting on none + prob_block3");
        for i in 0..3 {
            assert_eq!(
                1,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block3.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            let voter_block = utils::test_voter_block(
                prop_block2a.hash(), // Mining on 2a (because 3 hasnt showed up yet (fake))
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                vec![],
            );
            blockchain.insert_node(&voter_block);
        }
        for i in 3..6 {
            assert_eq!(
                1,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block3.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            let voter_block = utils::test_voter_block(
                prop_block3.hash(), // Mining on 3 after it showed up
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                vec![prop_block3.hash()],
            );
            blockchain.insert_node(&voter_block);
        }
        assert_eq!(55, blockchain.graph.node_count(), "Expecting 55 nodes");
        assert_eq!(111, blockchain.graph.edge_count(), "Expecting 176 edges");

        println!("Test 9:  Mining 2 tx block and 1 prop block referring the 2 tx blocks");
        unreferred_tx_block_index += 4;
        let tx_block_2: Vec<Block> =
            utils::test_tx_blocks_with_parent_hash(2, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_2.iter().cloned());
        // Add the tx blocks to blockchain
        for i in 0..2 {
            blockchain.insert_node(&tx_block_vec[unreferred_tx_block_index + i]);
        }
        let prop_block4 = utils::test_prop_block(
            blockchain.proposer_tree.best_block,
            tx_block_vec[14..16].iter().map(|x| x.hash()).collect(),
            vec![],
        ); // Referring 4 tx blocks + 1 prop_block
        blockchain.insert_node(&prop_block4);
        assert_eq!(
            prop_block4.hash(),
            blockchain.proposer_tree.best_block,
            "Proposer best block"
        );
        assert_eq!(58, blockchain.graph.node_count(), "Expecting 58 nodes");
        assert_eq!(116, blockchain.graph.edge_count(), "Expecting 186 edges");
        // Checking the number of unconfirmed tx blocks 2 of prop2a, 4 from prop3, and 2 from prop4.
        assert_eq!(8, blockchain.tx_blocks.not_in_ledger.len());

        println!("Test 10:  1-6 voter chains vote on prop4 and 6-10 voter blocks vote on prop3 and prop4" );
        //Storing voter_parents used in step 12 test
        for i in 0..10 {
            voter_best_blocks[i] = blockchain.voter_chains[i as usize].best_block.clone();
        }
        for i in 0..3 {
            assert_eq!(
                2,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block3.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            assert_eq!(
                prop_block4.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[1]
            );
            let voter_block = utils::test_voter_block(
                prop_block4.hash(), // Mining on 4
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                vec![prop_block3.hash(), prop_block4.hash()],
            );
            blockchain.insert_node(&voter_block);
        }

        // prop3 is confirmed. Unconfirmed tx blocks 2 from prop4.
        assert_eq!(2, blockchain.tx_blocks.not_in_ledger.len());

        for i in 3..6 {
            assert_eq!(
                1,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block4.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            let voter_block = utils::test_voter_block(
                prop_block4.hash(), // Mining on 4
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                vec![prop_block4.hash()],
            );
            blockchain.insert_node(&voter_block);
        }
        // prop4 is also confirmed.
        assert_eq!(0, blockchain.tx_blocks.not_in_ledger.len());

        for i in 6..10 {
            assert_eq!(
                2,
                blockchain.voter_chains[i as usize]
                    .get_unvoted_prop_blocks()
                    .len()
            );
            assert_eq!(
                prop_block3.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[0]
            );
            assert_eq!(
                prop_block4.hash(),
                blockchain.voter_chains[i as usize].get_unvoted_prop_blocks()[1]
            );
            let voter_block = utils::test_voter_block(
                prop_block4.hash(), // Mining on 3 after it showed up
                i as u16,
                blockchain.voter_chains[i as usize].best_block,
                vec![prop_block3.hash(), prop_block4.hash()],
            );
            blockchain.insert_node(&voter_block);
        }
        assert_eq!(68, blockchain.graph.node_count(), "Expecting 68 nodes");
        assert_eq!(160, blockchain.graph.edge_count(), "Expecting 240 edges");
        // Checking the voter chain growth
        for i in 0..6 {
            assert_eq!(4, blockchain.voter_chains[i as usize].best_level);
        }
        for i in 6..10 {
            assert_eq!(3, blockchain.voter_chains[i as usize].best_level);
        }

        println!("Test 11:  Checking get_proposer_parent()");
        assert_eq!(
            blockchain.get_proposer_parent(prop_block4.hash()),
            prop_block3.hash()
        );
        assert_eq!(
            blockchain.get_proposer_parent(tx_block_vec[14].hash()),
            prop_block3.hash()
        );
        for i in 0..10 {
            assert_eq!(
                blockchain.get_proposer_parent(blockchain.voter_chains[i as usize].best_block),
                prop_block4.hash()
            );
        }

        println!("Test 12:  Checking get_voter_parent()");
        for i in 0..10 {
            assert_eq!(
                blockchain.get_voter_parent(blockchain.voter_chains[i as usize].best_block),
                voter_best_blocks[i]
            );
        }

        println!("Test 13:  Checking get_votes_by_voter()");
        for i in 0..6 {
            let votes =
                blockchain.get_votes_by_voter(&blockchain.voter_chains[i as usize].best_block);
            let expected = vec![prop_block4.hash()];
            let matching = votes
                .iter()
                .zip(expected.iter())
                .filter(|&(a, b)| a == b)
                .count();
            assert_eq!(matching, expected.len());
        }
        for i in 6..10 {
            let mut votes =
                blockchain.get_votes_by_voter(&blockchain.voter_chains[i as usize].best_block);
            let mut expected = vec![prop_block3.hash(), prop_block4.hash()];
            votes.sort_by(|a, b| a.partial_cmp(b).unwrap());
            expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let matching = votes
                .iter()
                .zip(expected.iter())
                .filter(|&(a, b)| a == b)
                .count();
            assert_eq!(matching, expected.len());
        }

        println!("Test 14:  Checking get_referred_tx_blocks_ordered()");
        let mut referred_tx_blocks: Vec<H256> =
            blockchain.get_referred_tx_blocks_ordered(&prop_block4.hash());
        let mut expected: Vec<H256> = tx_block_vec[14..16].iter().map(|x| x.hash()).collect();
        referred_tx_blocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let matching = referred_tx_blocks
            .iter()
            .zip(expected.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, expected.len());
        // Check 2
        let mut referred_tx_blocks: Vec<H256> =
            blockchain.get_referred_tx_blocks_ordered(&prop_block2a.hash());
        let mut expected: Vec<H256> = tx_block_vec[5..8].iter().map(|x| x.hash()).collect();
        referred_tx_blocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let matching = referred_tx_blocks
            .iter()
            .zip(expected.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, expected.len());

        println!("Test 15:  Checking get_referred_prop_blocks() ");
        let mut referred_prop_blocks: Vec<H256> = blockchain
            .get_referred_prop_blocks(prop_block3.hash())
            .iter()
            .map(|x| x.0)
            .collect();
        let mut expected: Vec<H256> = vec![prop_block2b.hash()];
        referred_prop_blocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let matching = referred_prop_blocks
            .iter()
            .zip(expected.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, expected.len());

        println!("Test 16:  Checking get_votes_from_chain()");
        for i in 0..7 {
            let chain_votes = blockchain.get_votes_from_chain(i);
            let expected = vec![
                prop_block1a.hash(),
                prop_block2a.hash(),
                prop_block3.hash(),
                prop_block4.hash(),
            ];
            let matching = chain_votes
                .iter()
                .zip(expected.iter())
                .filter(|&(a, b)| a == b)
                .count();
            assert_eq!(matching, expected.len());
        }
        for i in 7..10 {
            let chain_votes = blockchain.get_votes_from_chain(i);
            let expected = vec![
                prop_block1a.hash(),
                prop_block2b.hash(),
                prop_block3.hash(),
                prop_block4.hash(),
            ];
            let matching = chain_votes
                .iter()
                .zip(expected.iter())
                .filter(|&(a, b)| a == b)
                .count();
            assert_eq!(matching, expected.len());
        }

        println!("Test 17:  Checking leader blocks for first four levels");
        let leader_block_sequence = blockchain.get_leader_block_sequence();
        assert_eq!(prop_block1a.hash(), leader_block_sequence[0]);
        assert_eq!(prop_block2a.hash(), leader_block_sequence[1]);
        assert_eq!(prop_block3.hash(), leader_block_sequence[2]);
        assert_eq!(prop_block4.hash(), leader_block_sequence[3]);
        assert_eq!(
            blockchain.proposer_node_data[&prop_block1a.hash()].leadership_status,
            ProposerStatus::Leader
        );
        assert_eq!(
            blockchain.proposer_node_data[&prop_block2a.hash()].leadership_status,
            ProposerStatus::Leader
        );
        assert_eq!(
            blockchain.proposer_node_data[&prop_block2b.hash()].leadership_status,
            ProposerStatus::NotLeaderAndConfirmed
        );
        assert_eq!(
            blockchain.proposer_node_data[&prop_block3.hash()].leadership_status,
            ProposerStatus::Leader
        );
        assert_eq!(
            blockchain.proposer_node_data[&prop_block4.hash()].leadership_status,
            ProposerStatus::Leader
        );

        println!("Test 18:  Checking NotLeaderUnconfirmed blocks for first four levels");
        assert_eq!(
            0,
            blockchain
                .get_unconfirmed_notleader_referred_proposer_blocks_prev_level(prop_block1a.hash())
                .len()
        );
        assert_eq!(
            0,
            blockchain
                .get_unconfirmed_notleader_referred_proposer_blocks_prev_level(prop_block2a.hash())
                .len()
        );
        assert_eq!(
            0,
            blockchain
                .get_unconfirmed_notleader_referred_proposer_blocks_prev_level(prop_block4.hash())
                .len()
        );

        println!("Test 19:  The ledger tx blocks");
        assert_eq!(16, blockchain.tx_blocks.ledger.len());
        let ordered_tx_blocks = blockchain.get_ordered_tx_blocks();
        for i in 0..16 {
            assert_eq!(ordered_tx_blocks[i], tx_block_vec[i].hash());
        }

        println!("Test 20: Mining 2 tx block and 1 prop block referring the 2 tx blocks");
        unreferred_tx_block_index += 2;
        let tx_block_2: Vec<Block> =
            utils::test_tx_blocks_with_parent_hash(2, blockchain.proposer_tree.best_block);
        tx_block_vec.extend(tx_block_2.iter().cloned());
        // Add the tx blocks to blockchain
        for i in 0..2 {
            blockchain.insert_node(&tx_block_vec[unreferred_tx_block_index + i]);
        }
        let prop_block5 = utils::test_prop_block(
            blockchain.proposer_tree.best_block,
            tx_block_vec[16..18].iter().map(|x| x.hash()).collect(),
            vec![],
        ); // Referring 4 tx blocks + 1 prop_block
        blockchain.insert_node(&prop_block5);

        assert_eq!(2, blockchain.tx_blocks.not_in_ledger.len());

        println!("Test 21: 51% attack - Changing level 1 leader ");

        let old_leader_blocks = blockchain.get_leader_block_sequence();
        //1. Mine another prop block at level 1
        let prop_block1b = utils::test_prop_block(
            proposer_genesis,
            tx_block_vec[0..5].iter().map(|x| x.hash()).collect(),
            vec![],
        );
        blockchain.insert_node(&prop_block1b);

        // Forking from genesis with length 5 sized voter chain on first 6 voter chains.
        for i in 0..10 {
            let voter_block1 = utils::test_voter_block(
                prop_block1b.hash(),
                i as u16,
                voter_genesis_blocks[i as usize],
                vec![prop_block1b.hash()],
            );
            blockchain.insert_node(&voter_block1);

            let voter_block2 = utils::test_voter_block(
                prop_block2a.hash(),
                i as u16,
                voter_block1.hash(),
                vec![prop_block2a.hash()],
            );
            blockchain.insert_node(&voter_block2);

            let voter_block3 = utils::test_voter_block(
                prop_block3.hash(),
                i as u16,
                voter_block2.hash(),
                vec![prop_block3.hash()],
            );
            blockchain.insert_node(&voter_block3);

            let voter_block4 =
                utils::test_voter_block(prop_block3.hash(), i as u16, voter_block3.hash(), vec![]);
            blockchain.insert_node(&voter_block4);

            let voter_block5 = utils::test_voter_block(
                prop_block4.hash(),
                i as u16,
                voter_block4.hash(),
                vec![prop_block4.hash()],
            );

            blockchain.insert_node(&voter_block5);

            // Check of the longest chain has switched
            assert_eq!(
                blockchain.voter_chains[i as usize].best_block,
                voter_block5.hash()
            );

            if i <= 3 {
                // Make sure that leader blocks have not changed
                let leader_blocks = blockchain.get_leader_block_sequence();
                for j in 0..leader_blocks.len() {
                    assert_eq!(leader_blocks[j], old_leader_blocks[j]);
                }
            }

            if i == 4 {
                // The first level has a tie in votes, so it has no leader yet
                assert_eq!(0, blockchain.get_leader_block_sequence().len());
            }
            if i >= 5 {
                // Make sure that only level 1 leader blocks hash changed
                let leader_blocks = blockchain.get_leader_block_sequence();
                assert_eq!(leader_blocks[0], prop_block1b.hash());
                for j in 1..leader_blocks.len() {
                    assert_eq!(leader_blocks[j], old_leader_blocks[j]);
                }
            }
        }
    }

    #[test]
    fn proposer_block_ordering() {
        pub const NUM_VOTER_CHAINS: u16 = 10;
        let _rng = rand::thread_rng();
        // Initialize a blockchain with 10 voter chains.
        let (state_update_sink, state_update_source) = mpsc::channel();
        let mut blockchain = BlockChain::new(NUM_VOTER_CHAINS, state_update_sink);

        // Store the parent blocks to mine on voter trees.
        let _voter_best_blocks: Vec<H256> = (0..NUM_VOTER_CHAINS)
            .map(|i| blockchain.voter_chains[i as usize].best_block)
            .collect(); // Currently the voter genesis blocks.

        // Maintains the list of tx blocks.
        let _tx_block_vec: Vec<Block> = vec![];
        let _unreferred_tx_block_index = 0;

        assert_eq!(
            11,
            blockchain.graph.node_count(),
            "Expecting 11 nodes corresponding to 11 genesis blocks"
        );
        assert_eq!(0, blockchain.graph.edge_count(), "Expecting 0 edges");

        // Adding 6 prop blocks with notleader status
        let prop_block1a =
            utils::test_prop_block(blockchain.proposer_tree.best_block, vec![], vec![]);
        //        println!("P-1: {}", prop_block1a.hash());
        blockchain.insert_node(&prop_block1a);

        let prop_block2a = utils::test_prop_block(prop_block1a.hash(), vec![], vec![]);
        //        println!("P-2a: {}", prop_block2a.hash());
        blockchain.insert_node(&prop_block2a);
        let prop_block2b = utils::test_prop_block(prop_block1a.hash(), vec![], vec![]);
        //        println!("P-2b: {}", prop_block2b.hash());
        blockchain.insert_node(&prop_block2b);
        let prop_block2c = utils::test_prop_block(prop_block1a.hash(), vec![], vec![]);
        //        println!("P-2c: {}", prop_block2c.hash());
        blockchain.insert_node(&prop_block2c);

        let prop_block3a = utils::test_prop_block(
            blockchain.proposer_tree.best_block,
            vec![],
            vec![prop_block2b.hash()],
        );
        //        println!("P-3a: {}", prop_block3a.hash());
        blockchain.insert_node(&prop_block3a);
        let prop_block3b = utils::test_prop_block(prop_block2b.hash(), vec![], vec![]);
        //        println!("P-3b: {}", prop_block3b.hash());
        blockchain.insert_node(&prop_block3b);

        let prop_block4a = utils::test_prop_block(
            prop_block3b.hash(),
            vec![],
            vec![prop_block2a.hash(), prop_block3a.hash()],
        );
        //        println!("P-4a: {}", prop_block4a.hash());
        blockchain.insert_node(&prop_block4a);
        let prop_block4b = utils::test_prop_block(prop_block3b.hash(), vec![], vec![]);
        //        println!("P-4b: {}", prop_block4b.hash());
        blockchain.insert_node(&prop_block4b);

        let prop_block5a = utils::test_prop_block(
            prop_block4b.hash(),
            vec![],
            vec![
                prop_block2c.hash(),
                prop_block2a.hash(),
                prop_block4a.hash(),
                prop_block3a.hash(),
            ],
        );
        //        println!("P-5a: {}", prop_block5a.hash());
        blockchain.insert_node(&prop_block5a);

        /*
                    _____
                    | 1 |<===================\\
                    |___|<========||         ||
                      ||          ||         ||
                    __||_       __||_       _||__
             /----->| 2a|   /-->| 2b|       | 2c|
             |  /-->|___|  /    |___|       |___|<--\
             |  |     ||  /       ||                |
             |  |   __||_/      __||_               |
             |  |   | 3a|       | 3b|               |
             |  |   |___|       |___|<========\\    |
             |  |     |           ||          ||    |
             |  |     |         __||__      __||_   |
             |__|_____|_________| 4a|       | 4b|   |
                |     |         |___|       |___|   |
                |     |   --------|           ||    |
                |   __|__/                    ||    |
                \---| 5 |=====================//    |
                    |___|---------------------------|


        */

        // Changing it to notleader status ONLY for testing
        blockchain
            .proposer_node_data
            .get_mut(&prop_block1a.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block2a.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block2b.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block2c.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block3a.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block3b.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block4a.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block4b.hash())
            .unwrap()
            .give_not_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block5a.hash())
            .unwrap()
            .give_not_leader_status();

        println!("Test 2:   Checking the order of get_unconfirmed_notleader_referred_proposer_blocks_prev_level()");
        let prop_block_2a_ref = blockchain
            .get_unconfirmed_notleader_referred_proposer_blocks_prev_level(prop_block2a.hash());
        assert_eq!(1, prop_block_2a_ref.len());
        assert_eq!(prop_block1a.hash(), prop_block_2a_ref[0].0);

        let prop_block_3a_ref = blockchain
            .get_unconfirmed_notleader_referred_proposer_blocks_prev_level(prop_block3a.hash());
        assert_eq!(2, prop_block_3a_ref.len());
        assert_eq!(prop_block2a.hash(), prop_block_3a_ref[0].0);
        assert_eq!(prop_block2b.hash(), prop_block_3a_ref[1].0);

        let prop_block_3b_ref = blockchain
            .get_unconfirmed_notleader_referred_proposer_blocks_prev_level(prop_block3b.hash());
        assert_eq!(1, prop_block_3b_ref.len());
        assert_eq!(prop_block2b.hash(), prop_block_3b_ref[0].0);

        let prop_block_4a_ref = blockchain
            .get_unconfirmed_notleader_referred_proposer_blocks_prev_level(prop_block4a.hash());
        assert_eq!(3, prop_block_4a_ref.len());
        assert_eq!(prop_block3b.hash(), prop_block_4a_ref[0].0);
        assert_eq!(prop_block2a.hash(), prop_block_4a_ref[1].0);
        assert_eq!(prop_block3a.hash(), prop_block_4a_ref[2].0);

        println!(
            "Test 3:   Checking the order of get_unconfirmed_notleader_referred_proposer_blocks()"
        );
        let prop_block_2a_ref =
            blockchain.get_unconfirmed_notleader_referred_proposer_blocks(prop_block2a.hash());
        assert_eq!(2, prop_block_2a_ref.len());
        // The expected order
        assert_eq!(prop_block1a.hash(), prop_block_2a_ref[0]);
        assert_eq!(prop_block2a.hash(), prop_block_2a_ref[1]);

        let prop_block_3a_ref =
            blockchain.get_unconfirmed_notleader_referred_proposer_blocks(prop_block3a.hash());
        assert_eq!(4, prop_block_3a_ref.len());

        // The expected order
        assert_eq!(prop_block1a.hash(), prop_block_3a_ref[0]);
        assert_eq!(prop_block2a.hash(), prop_block_3a_ref[1]);
        assert_eq!(prop_block2b.hash(), prop_block_3a_ref[2]);
        assert_eq!(prop_block3a.hash(), prop_block_3a_ref[3]);

        let prop_block_4a_ref =
            blockchain.get_unconfirmed_notleader_referred_proposer_blocks(prop_block4a.hash());
        assert_eq!(6, prop_block_4a_ref.len());
        assert_eq!(prop_block1a.hash(), prop_block_4a_ref[0]);
        assert_eq!(prop_block2b.hash(), prop_block_4a_ref[1]);
        assert_eq!(prop_block2a.hash(), prop_block_4a_ref[2]);
        assert_eq!(prop_block3b.hash(), prop_block_4a_ref[3]);
        assert_eq!(prop_block3a.hash(), prop_block_4a_ref[4]);
        assert_eq!(prop_block4a.hash(), prop_block_4a_ref[5]);

        //        println!(" Here ");
        let prop_block_5a_ref =
            blockchain.get_unconfirmed_notleader_referred_proposer_blocks(prop_block5a.hash());
        assert_eq!(prop_block2b.hash(), prop_block_5a_ref[1], "1");
        assert_eq!(prop_block2c.hash(), prop_block_5a_ref[2], "2");
        assert_eq!(prop_block2a.hash(), prop_block_5a_ref[3], "3");
        assert_eq!(prop_block3b.hash(), prop_block_5a_ref[4], "4");
        assert_eq!(prop_block3a.hash(), prop_block_5a_ref[5], "5");
        assert_eq!(prop_block4b.hash(), prop_block_5a_ref[6], "6");
        assert_eq!(prop_block4a.hash(), prop_block_5a_ref[7], "7");
        assert_eq!(prop_block5a.hash(), prop_block_5a_ref[8], "8");

        // Making 1, 2a leaders
        blockchain
            .proposer_node_data
            .get_mut(&prop_block1a.hash())
            .unwrap()
            .give_leader_status();
        blockchain
            .proposer_node_data
            .get_mut(&prop_block2a.hash())
            .unwrap()
            .give_leader_status();
        let prop_block_5a_ref =
            blockchain.get_unconfirmed_notleader_referred_proposer_blocks(prop_block5a.hash());
        assert_eq!(prop_block2b.hash(), prop_block_5a_ref[0], "1");
        assert_eq!(prop_block2c.hash(), prop_block_5a_ref[1], "2");
        assert_eq!(prop_block3b.hash(), prop_block_5a_ref[2], "4");
        assert_eq!(prop_block3a.hash(), prop_block_5a_ref[3], "5");
        assert_eq!(prop_block4b.hash(), prop_block_5a_ref[4], "6");
        assert_eq!(prop_block4a.hash(), prop_block_5a_ref[5], "7");
        assert_eq!(prop_block5a.hash(), prop_block_5a_ref[6], "8");
    }
}
