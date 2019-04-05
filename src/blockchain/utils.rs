use crate::block::header::Header;
use crate::block::proposer::Content as Proposer_Content;
use crate::block::transaction::Content as Tx_Content;
use crate::block::voter::Content as Voter_Content;
use crate::block::{Block, Content};

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;

use crate::block::generator as block_generator;
use crate::crypto::generator as crypto_generator;
use crate::transaction::generator as tx_generator;

use rand::{Rng, RngCore};
use std::cmp;
use std::cmp::Ordering;

pub fn lcb_from_vote_depths(votes: Vec<u32>) -> f32 {
    let answer: f32 = votes.len() as f32;
    return answer; //todo: Apply the confirmation logic from the paper
}

#[derive(Eq, PartialEq, Clone)]
pub struct PropOrderingHelper {
    pub level: u32,
    pub position: Vec<u32>,
}

impl Ord for PropOrderingHelper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.level < other.level {
            return Ordering::Less;
        } else if self.level > other.level {
            return Ordering::Greater;
        }

        // If they have same levels then use the position
        let len = cmp::min(self.position.len(), other.position.len());
        for i in 0..len {
            if self.position[i] < other.position[i] {
                return Ordering::Less;
            } else if self.position[i] > other.position[i] {
                return Ordering::Greater;
            }
        }
        if self.position.len() == other.position.len() {
            return Ordering::Equal;
        }
        panic!("This is not supposed to happen");
    }
}

impl PartialOrd for PropOrderingHelper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PropOrderingHelper {
    pub fn new(level: u32, position: Vec<u32>) -> Self {
        return PropOrderingHelper { level, position };
    }
}

/*
 Test utils
*/

/// Generates a random tx_block with the given parent_hash. Only used by 'tx_blocks_with_parent_hash' fn.
fn test_tx_block_with_parent_hash(parent_hash: H256) -> Block {
    let mut tx_block = block_generator::tx_block();
    tx_block.header.parent_hash = parent_hash;
    return tx_block;
}

/// Returns 'num' random tx blocks with the same parent hash.
pub fn test_tx_blocks_with_parent_hash(num: u32, parent_hash: H256) -> Vec<Block> {
    return (0..num)
        .map(|_| test_tx_block_with_parent_hash(parent_hash))
        .collect();
}

/// Returns proposer block which has parent_hash, tx_blocks_hashes and pro_block_hashes. Everything other field is random.
pub fn test_prop_block(
    parent_hash: H256,
    transaction_block_hashes: Vec<H256>,
    proposer_block_hashes: Vec<H256>,
) -> Block {
    let mut header = block_generator::header(); // Random header
    header.parent_hash = parent_hash;
    let proposer_content = Proposer_Content {
        transaction_block_hashes,
        proposer_block_hashes,
    };
    let content = Content::Proposer(proposer_content);
    let sortition_proof: Vec<H256> = (0..10).map(|_| crypto_generator::h256()).collect();
    return Block {
        header,
        content,
        sortition_proof,
    };
}

/// Returns voter block which has parent_hash, chain_number, voter_parent_hash and proposer_block_votes
/// Everything other field is random.
pub fn test_voter_block(
    parent_hash: H256,
    chain_number: u16,
    voter_parent_hash: H256,
    proposer_block_votes: Vec<H256>,
) -> Block {
    let mut header = block_generator::header(); // Random header
    header.parent_hash = parent_hash;
    let voter_content = Voter_Content {
        chain_number,
        voter_parent_hash,
        proposer_block_votes,
    };
    let content = Content::Voter(voter_content);
    let sortition_proof: Vec<H256> = (0..10).map(|_| crypto_generator::h256()).collect();
    return Block {
        header,
        content,
        sortition_proof,
    };
}
