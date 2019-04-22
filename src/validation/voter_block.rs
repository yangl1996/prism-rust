use super::super::config;
use super::data_availability::{self, get_block};
use super::*;
use crate::block::Block;
use crate::block::Content;
use crate::blockchain::BlockChain;
use crate::blockchain::utils::get_voter_genesis_hash;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use std::sync::{Arc, Mutex};
use super::BlockResult;

fn validate(block: &Block, blockchain: &Mutex<BlockChain>, blockdb: &BlockDatabase) -> BlockResult {
    let content = match &block.content {
        Content::Voter(content) => content,
        _ => panic!("Wrong type"),
    };

    // Check 1: If the chain number is valid
    let chain_number = content.chain_number;
    if chain_number < 0 || chain_number > config::NUM_VOTER_CHAINS {
        return BlockResult::Fail;
    }
    let mut missing_blocks: Vec<H256> = vec![];

    // Check 2. If the parent voter block is available
    let mut latest_level_voted_by_ancestor: usize = 0;
    let voter_parent =
        get_block(content.voter_parent_hash, blockchain, blockdb);
    match voter_parent {
        Ok(voter_parent_block) => {
            latest_level_voted_by_ancestor = latest_level_voted_on_chain(&voter_parent_block,
                                                                         blockchain,
                                                                         blockdb);
        }
        Err(data_availability::Error::MissingInDB) => {
            missing_blocks.push(content.voter_parent_hash);
        }
        Err(data_availability::Error::MissingInBlockchain) => {
            missing_blocks.push(content.voter_parent_hash);
        }
    }

    // Check 3. If all voted proposer blocks are available and are on continuous level from
    // latest_level_voted_by_ancestor onwards
    for (index, proposer_vote) in content.proposer_block_votes.iter().enumerate() {
        let proposer_block =
            get_block(*proposer_vote, blockchain, blockdb);
        match proposer_block {
            Ok(block) => {
                let blockchain_l = blockchain.lock().unwrap();
                let level = blockchain_l.prop_node_data(&block.hash()).level as usize;
                drop(blockchain_l);
                if level != index + 1 + latest_level_voted_by_ancestor {
                    //The votes are not on continuous levels and hence the block is invalid.
                    return BlockResult::Fail;
                }
            }
            Err(data_availability::Error::MissingInDB) => {
                missing_blocks.push(*proposer_vote);
            }
            Err(data_availability::Error::MissingInBlockchain) => {
                missing_blocks.push(*proposer_vote);
            }
        }
    }

    //Final result: If all the data is available then the block is valid
    if missing_blocks.len() == 0 {
        return BlockResult::Pass;
    } else {
        return BlockResult::MissingReferences(missing_blocks);
    }
}

/// Returns the last proposer level voted by the voter chain until 'voter_block'
fn latest_level_voted_on_chain(
    voter_block: &Block,
    blockchain: &Mutex<BlockChain>,
    blockdb: &BlockDatabase,
) -> usize {
    let content = match &voter_block.content {
        Content::Voter(content) => content,
        _ => panic!("Wrong type"),
    };

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
        let voter_parent = get_block(content.voter_parent_hash, blockchain, blockdb);
        match voter_parent {
            Ok(voter_block_inner) => {
                return latest_level_voted_on_chain(&voter_block_inner, blockchain, blockdb);
            }
            _ => panic!("This shouldn't have happened! The parent block should be there in both db and bc."),
        }
    }
}

// TODO: Add tests
