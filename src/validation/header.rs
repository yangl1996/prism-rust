use super::super::config;
use super::data_availability::{self, get_block};
use super::*;
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use bigint::uint::U256;
use std::sync::{Arc, Mutex};

pub fn check_pow(block: &Block, proposer_range: (U256, U256), voter_ranges: Vec<(U256, U256)>, transaction_range: (U256, U256)) -> BlockResult {
    // TODO: avoid doing those conversions
    let hash: [u8; 32] = (&block.header.hash()).into();
    let hash = U256::from_big_endian(&hash);
    match &block.content {
        Content::Proposer(_) => {
            if proposer_range.0 <= hash && hash <= proposer_range.1 {
                return BlockResult::Pass;
            }
            return BlockResult::Fail;
        }
        Content::Transaction(_) => {
            if transaction_range.0 <= hash && hash <= transaction_range.1 {
                return BlockResult::Pass;
            }
            return BlockResult::Fail;
        }
        Content::Voter(c) => {
            let voter_range = voter_ranges[c.chain_number as usize];
            if voter_range.0 <= hash && hash <= voter_range.1 {
                return BlockResult::Pass;
            }
            return BlockResult::Fail;
        }
    }
}

pub fn check_sortition(block: &Block, blockchain: &Mutex<BlockChain>, blockdb: &BlockDatabase) -> BlockResult {
    // 1. Check parent proposer block is available
    let proposer_parent_hash = block.header.parent_hash;
    let proposer_parent = data_availability::get_block(
        proposer_parent_hash,
        blockchain,
        blockdb,
    );
    match proposer_parent {
        Ok(block) => {
            // good, do nothing
        }
        Err(_) => {
            return BlockResult::MissingReferences(vec![proposer_parent_hash]);
        }
    }

    // 2. Check sortition proof
    if check_sortition_proof(block) {
        return BlockResult::Pass;
    }

    unimplemented!();
}

/// Checks the sortition logic
pub fn check_sortition_proof(block: &Block) -> bool {
    //1. First check if hash(content) =? content_root
    match &block.content {
        Content::Transaction(content) => {
            if content.hash() != block.header.content_root {
                return false;
            }
        }
        Content::Proposer(content) => {
            if content.hash() != block.header.content_root {
                return false;
            }
        }
        Content::Voter(content) => {
            if content.hash() != block.header.content_root {
                return false;
            }
        }
    }

    //2. Check sortition merkle proof
    // function(block.hash, block.header.content_root, block.sortition_proof
    // TODO: To add the function
    return true;
}

//TODO: Add timestamp rule?
