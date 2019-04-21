/*
Validation for blocks.
*/
use crate::block::{Block, Content};
use crate::crypto::hash::Hashable;
use bigint::uint::U256;
use std::sync::{Arc, Mutex};

pub trait BlockValidator {
    fn is_valid(&self, block: &Block) -> bool;
}

/// Checks PoW difficulty
pub struct DifficultyValidator {
    proposer_range: (U256, U256),
    voter_ranges: Vec<(U256, U256)>,
    transaction_range: (U256, U256),
}
impl BlockValidator for DifficultyValidator {
    fn is_valid(&self, block: &Block) -> bool {
        let hash: [u8; 32] = (&block.header.hash()).into();
        let hash = U256::from_big_endian(&hash);
        match &block.content {
            Content::Proposer(_) => {
                return self.proposer_range.0 <= hash && hash <= self.proposer_range.1;
            }
            Content::Transaction(_) => {
                return self.transaction_range.0 <= hash && hash <= self.transaction_range.1;
            }
            Content::Voter(c) => {
                let voter_range = self.voter_ranges[c.chain_number as usize];
                return voter_range.0 <= hash && hash <= voter_range.1;
            }
        }
    }
}

#[derive(Default)]
pub struct ValidatorCollection {
    list: Vec<Box<BlockValidator>>,
}

impl BlockValidator for ValidatorCollection {
    fn is_valid(&self, block: &Block) -> bool {
        self.list.iter().all(|validator| validator.is_valid(block))
    }
}

impl ValidatorCollection {
    pub fn new(list: Vec<Box<BlockValidator>>) -> Self {
        Self { list }
    }
}
#[cfg(test)]

// TODO: Add more tests
pub mod tests {}
