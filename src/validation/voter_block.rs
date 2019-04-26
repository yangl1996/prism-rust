use super::super::config;
use super::check_block_exist;
use crate::block::voter::Content;
use crate::block::Block;
use crate::block::Content as BlockContent;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use std::sync::Mutex;

pub fn get_missing_references(
    content: &Content,
    blockchain: &Mutex<BlockChain>,
    blockdb: &BlockDatabase,
) -> Vec<H256> {
    let mut missing_blocks = vec![];

    // check the voter parent
    let voter_parent = check_block_exist(content.voter_parent, blockchain, blockdb);
    if !(voter_parent.0 && voter_parent.1) {
        missing_blocks.push(content.voter_parent);
    }

    // check the votes
    for prop_hash in content.votes.iter() {
        let avail = check_block_exist(*prop_hash, blockchain, blockdb);
        if !(avail.0 && avail.1) {
            missing_blocks.push(*prop_hash);
        }
    }

    return missing_blocks;
}

pub fn check_chain_number(content: &Content) -> bool {
    let chain_num = content.chain_number;
    if chain_num > config::NUM_VOTER_CHAINS {
        return false;
    } else {
        return true;
    }
}

pub fn check_levels_voted(
    content: &Content,
    blockchain: &Mutex<BlockChain>,
    blockdb: &BlockDatabase,
) -> bool {
    // get the deepest block voted by our parent
    let parent_block = blockdb.get(content.voter_parent).unwrap().unwrap();
    let last_voted_level = latest_level_voted_on_chain(&parent_block, blockchain, blockdb);

    // check whether the votes are continuous, and starts at the next unvoted level
    for (index, proposer_vote) in content.votes.iter().enumerate() {
        let voted = blockdb.get(*proposer_vote).unwrap().unwrap();
        let level = blockchain
            .lock()
            .unwrap()
            .prop_node_data(&voted.hash())
            .level as usize;
        if level != index + 1 + last_voted_level {
            return false;
        }
    }

    return true;
}

/// Get the deepest proposer level voted by this chain, until the given voter block.
fn latest_level_voted_on_chain(
    voter_block: &Block,
    blockchain: &Mutex<BlockChain>,
    blockdb: &BlockDatabase,
) -> usize {
    let content = match &voter_block.content {
        BlockContent::Voter(content) => content,
        _ => panic!("Wrong type"),
    };

    let voter_genesis_hash = VOTER_GENESIS_HASHES[content.chain_number as usize];

    if voter_block.hash() == voter_genesis_hash {
        // if the voter block is the genesis block
        return 0;
    } else if content.votes.len() > 0 {
        // if this block voted for some blocks, then just return the deepest level among them
        let latest_prop_voted = content.votes.last().unwrap();
        return blockchain
            .lock()
            .unwrap()
            .prop_node_data(latest_prop_voted)
            .level as usize;
    } else {
        // if this block voted for zero block, then look for its parent
        let parent = blockdb.get(content.voter_parent).unwrap().unwrap();
        return latest_level_voted_on_chain(&parent, blockchain, blockdb);
    }
}

// TODO: Add tests
