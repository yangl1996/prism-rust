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
    let should_vote = blockchain.unvoted_proposer(&content.voter_parent, &parent).unwrap();

    if content.votes.len() != should_vote.len() { return false; }

    content.votes.iter().zip(should_vote.into_iter()).all(|(x,y)|*x == y)
}

