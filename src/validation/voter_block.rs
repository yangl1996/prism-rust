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

    // check the votes
    for prop_hash in content.votes.iter() {
        let avail = check_proposer_block_exists(*prop_hash, blockchain);
        if !avail {
            missing_blocks.push(*prop_hash);
        }
    }

    return missing_blocks;
}

pub fn check_chain_number(parent: &H256, content: &Content, blockchain: &BlockChain) -> bool {
    let chain_num = blockchain
        .voter_chain_number(parent)
        .unwrap();
    chain_num == content.chain_number
}

pub fn check_levels_voted(content: &Content, blockchain: &BlockChain, parent: &H256) -> bool {
    /*
    let mut start = blockchain
        .deepest_voted_level(parent)
        .unwrap(); //need to be +1
    let end = blockchain.proposer_level(parent).unwrap();

    if start > end {
        return false;
    } //end < start means incorrect parent level
    if content.votes.len() != (end - start) as usize {
        return false;
    } //
    for vote in content.votes.iter() {
        start += 1;
        if start != blockchain.proposer_level(vote).unwrap() {
            return false;
        }
    }
    */
    true
}
