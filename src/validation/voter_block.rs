use super::super::config;
use super::{check_proposer_block_exists, check_voter_block_exists};
use crate::block::voter::Content;
use crate::block::Block;
use crate::block::Content as BlockContent;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};

pub fn get_missing_references(
    content: &Content,
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
) -> Vec<H256> {
    let mut missing_blocks = vec![];

    // check the voter parent
    let voter_parent = check_voter_block_exists(content.voter_parent, blockdb, blockchain);
    if !voter_parent {
        missing_blocks.push(content.voter_parent);
    }

    // check the votes
    for prop_hash in content.votes.iter() {
        let avail = check_proposer_block_exists(*prop_hash, blockdb, blockchain);
        if !avail {
            missing_blocks.push(*prop_hash);
        }
    }

    return missing_blocks;
}

pub fn check_chain_number(content: &Content, blockchain: &BlockChain) -> bool {
    let chain_num = blockchain.voter_chain_number(&content.voter_parent).unwrap();
    chain_num == content.chain_number
}

pub fn check_levels_voted(
    content: &Content,
    blockchain: &BlockChain,
    parent: &H256
) -> bool {
    let should_vote = blockchain.unvoted_proposer(&content.voter_parent, &parent).unwrap();//latest_level_voted_on_chain(&parent_block, blockchain, blockdb);

    if content.votes.len() != should_vote.len() { return false; }

    content.votes.iter().zip(should_vote.into_iter()).all(|(x,y)|*x == y)
}

///// Get the deepest proposer level voted by this chain, until the given voter block.
//fn latest_level_voted_on_chain(
//    voter_block: &Block,
//    blockchain: &BlockChain,
//    blockdb: &BlockDatabase,
//) -> usize {
//    let content = match &voter_block.content {
//        BlockContent::Voter(content) => content,
//        _ => panic!("Wrong type"),
//    };
//
//    let voter_genesis_hash = VOTER_GENESIS_HASHES[content.chain_number as usize];
//
//    if voter_block.hash() == voter_genesis_hash {
//        // if the voter block is the genesis block
//        return 0;
//    } else if content.votes.len() > 0 {
//        // if this block voted for some blocks, then just return the deepest level among them
//        let latest_prop_voted = content.votes.last().unwrap();
//        return blockchain.proposer_level(latest_prop_voted).unwrap() as usize;
//    } else {
//        // if this block voted for zero block, then look for its parent
//        let parent = blockdb.get(&content.voter_parent).unwrap().unwrap();
//        return latest_level_voted_on_chain(&parent, blockchain, blockdb);
//    }
//}

// TODO: Add tests
