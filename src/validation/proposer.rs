/*
Validation for blocks and transactions.
*/


use crate::block::{Block};
use crate::crypto::hash::{Hashable,H256};
use std::collections::HashSet;

pub struct ProposerBlockValidator {
    // Database of known blocks
    pub known_blocks: HashSet<H256>
    // TODO: change this to the appropriate data structure
}

impl ProposerBlockValidator {
    pub fn proposer_blk_valid(&self, block: Block) -> bool {
        // Get hash of block
        let block_hash: H256 = block.hash();
        
        // 1. Check if block is a duplicate
        if self.is_duplicate(&block_hash) {
            return false;
        }
        // 2. Check if block contains a nonzero number of reflinks
        if self.empty_reflinks(&block) {
            return false;
        }

        // 3. Check if coinbase is invalid
        if !self.is_coinbase_valid(&block) {
            return false;
        }

        // 4. Check that pow is valid
        if !self.pow_valid(&block) {
            return false;
        }
        
        // 5. Check Merkle proof
        if !self.is_merkle_proof_valid(&block) {
            return false;
        }
        return true;
    }

    pub fn is_duplicate(&self, block_hash: &H256) -> bool {
        // Checks if we already have a copy of this block in storage
        // TODO: Replace with blocktree
        return self.known_blocks.contains(block_hash);
    }

    pub fn empty_reflinks(&self, block: &Block) -> bool {
        // Checks if the reflinks in the block are empty
        // TODO: Fill in with real code
        return false;
    }

    pub fn is_coinbase_valid(&self, block: &Block) -> bool {
        // TODO: replace with coinbase transaction validity check
        return true;
    }

    pub fn pow_valid(&self, block: &Block) -> bool {
        return true;
    }

    pub fn is_merkle_proof_valid(&self, block: &Block) -> bool {

        return true;
    }
}