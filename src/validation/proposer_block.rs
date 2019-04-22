use crate::block::proposer::Content;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::H256;
use std::sync::Mutex;
use super::check_block_exist;

pub fn get_missing_references(content: &Content, blockchain: &Mutex<BlockChain>, blockdb: &BlockDatabase) -> Vec<H256> {
    let mut missing_blocks: Vec<H256> = vec![];

    // check whether the tx block referred are present
    for tx_block_hash in content.transaction_block_hashes.iter() {
        let tx_block = check_block_exist(*tx_block_hash, blockchain, blockdb);
        if !(tx_block.0 && tx_block.1) {
            missing_blocks.push(*tx_block_hash);
        }
    }

    // check whether the proposer blocks referred are present
    for prop_block_hash in content.proposer_block_hashes.iter() {
        let prop_block = check_block_exist(*prop_block_hash, blockchain, blockdb);
        if !(prop_block.0 && prop_block.1) {
            missing_blocks.push(*prop_block_hash);
        }
    }

    return missing_blocks;
}

// TODO: Add tests
