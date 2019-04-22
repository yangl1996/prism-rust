use super::super::config;
use super::data_availability::*;
use super::*;
use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockchain::utils::get_voter_genesis_hash;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use std::sync::{Arc, Mutex};

/// Checks data availability and prism logic
pub struct VoterBlockRule {
    blockchain: Arc<Mutex<BlockChain>>,
    block_db: Arc<BlockDatabase>,
}
impl BlockRule for VoterBlockRule {
    fn result(&self, block: &Block) -> BlockRuleResult {
        let content = block.get_voter_content();
        let chain_number = content.chain_number;
        //Check 1. If the chain number is valid
        if chain_number < 0 || chain_number > config::NUM_VOTER_CHAINS {
            // Invalid chain number
            return BlockRuleResult::False;
        }
        let mut blocks_not_in_db: Vec<H256> = vec![];
        let mut blocks_not_in_blockchain: Vec<H256> = vec![];

        //Check 2. If the parent voter block is available
        let mut latest_level_voted_by_ancestor: usize = 0;
        let voter_parent =
            get_available_block(content.voter_parent_hash, &self.blockchain, &self.block_db);
        match voter_parent {
            BlockDataAvailability::NotInDB => {
                // The voter parent should be requested from the network
                blocks_not_in_db.push(content.voter_parent_hash);
            }
            BlockDataAvailability::NotInBlockchain => {
                // The voter parent should be added to the blockchain first
                blocks_not_in_blockchain.push(content.voter_parent_hash);
            }
            BlockDataAvailability::Block(voter_parent_block) => {
                latest_level_voted_by_ancestor = latest_level_voted_on_chain(
                    &voter_parent_block,
                    &self.blockchain,
                    &self.block_db,
                );
            }
        }

        //Check 3. If all voted proposer blocks are available and are on continuous level from
        //   latest_level_voted_by_ancestor onwards
        for (index, proposer_vote) in content.proposer_block_votes.iter().enumerate() {
            let proposer_block =
                get_available_block(*proposer_vote, &self.blockchain, &self.block_db);
            match proposer_block {
                BlockDataAvailability::NotInDB => {
                    // The voter parent should be requested from the network
                    blocks_not_in_db.push(*proposer_vote);
                }
                BlockDataAvailability::NotInBlockchain => {
                    // The voter parent should be added to the blockchain first
                    blocks_not_in_blockchain.push(*proposer_vote);
                }
                BlockDataAvailability::Block(block) => {
                    let blockchain_l = self.blockchain.lock().unwrap();
                    let level = blockchain_l.prop_node_data(&block.hash()).level as usize;
                    drop(blockchain_l);
                    if level != index + 1 + latest_level_voted_by_ancestor {
                        //The votes are not on continuous levels and hence the block is invalid.
                        return BlockRuleResult::False;
                    }
                }
            }
        }

        //Final result: If all the data is available then the block is valid
        if blocks_not_in_db.len() == 0 && blocks_not_in_blockchain.len() == 0 {
            return BlockRuleResult::True;
        } else {
            return BlockRuleResult::MissingReferencesInDBandBC(
                blocks_not_in_db,
                blocks_not_in_blockchain,
            );
        }
    }
}

/// Returns the last proposer level voted by the voter chain until 'voter_block'
fn latest_level_voted_on_chain(
    voter_block: &Block,
    blockchain: &Mutex<BlockChain>,
    block_db: &BlockDatabase,
) -> usize {
    let content = voter_block.get_voter_content();
    let voter_genesis_hash = get_voter_genesis_hash(content.chain_number);
    // Base case
    if voter_block.hash() == voter_genesis_hash {
        return 0;
    } else if content.proposer_block_votes.len() > 0 {
        // If the block content has any votes, then return the latest voted level
        let latest_prop_voted = content.proposer_block_votes.last().unwrap();
        let blockchain_l = blockchain.lock().unwrap();
        return blockchain_l.prop_node_data(latest_prop_voted).level as usize;
    } else {
        // Else call the function on its voter parent block
        let voter_parent = get_available_block(content.voter_parent_hash, blockchain, block_db);
        match voter_parent {
            BlockDataAvailability::Block(voter_block_inner) => {
                return latest_level_voted_on_chain(&voter_block_inner, blockchain, block_db);
            }
            _ => panic!("This shouldn't have happened! The parent block should be there in both db and bc."),
        }
    }
}

// TODO: Add tests
