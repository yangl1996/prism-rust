use super::super::config;
use super::data_availability::{self, get_block};
use super::*;
use crate::block::Block;
use crate::block::Content;
use crate::blockchain::BlockChain;
use crate::blockchain::utils::get_proposer_genesis_hash;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use std::sync::{Arc, Mutex};
use super::BlockResult;

pub fn validate(block: &Block, blockchain: &Mutex<BlockChain>, blockdb: &BlockDatabase) -> BlockResult {
    let content = match &block.content {
        Content::Proposer(content) => content,
        _ => panic!("Wrong type"),
    };

    let mut missing_blocks: Vec<H256> = vec![];

    // check whether the tx block referred are present
    for tx_block_hash in content.transaction_block_hashes.iter() {
        let tx_block =
            get_block(*tx_block_hash, blockchain, blockdb);
        match tx_block {
            Ok(_) => {
                // good. do nothing
            }
            Err(data_availability::Error::MissingInDB) => {
                missing_blocks.push(*tx_block_hash);
            }
            Err(data_availability::Error::MissingInBlockchain) => {
                missing_blocks.push(*tx_block_hash);
            }
        }
    }

    // check whether the proposer blocks referred are present
    for prop_block_hash in content.proposer_block_hashes.iter() {
        let prop_block =
            get_block(*prop_block_hash, blockchain, blockdb);
        match prop_block {
            Ok(_) => {
                // good. do nothing
            }
            Err(data_availability::Error::MissingInDB) => {
                missing_blocks.push(*prop_block_hash);
            }
            Err(data_availability::Error::MissingInBlockchain) => {
                missing_blocks.push(*prop_block_hash);
            }
        }
    }

    if missing_blocks.len() == 0 {
        return BlockResult::Pass;
    } else {
        return BlockResult::MissingReferences(missing_blocks);
    }
}

// TODO: Add tests
