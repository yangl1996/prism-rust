use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use super::Block;
use super::Content as BlockContent;
use crate::config::*;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    /// Voter chain id
    pub chain_number: u16,
    /// Hash of the parent voter block.
    pub voter_parent_hash: H256,
    /// List of votes on proposer blocks.
    pub proposer_block_votes: Vec<H256>,
}

impl Content {
    pub fn new(
        chain_number: u16,
        voter_parent_hash: H256,
        proposer_block_votes: Vec<H256>,
    ) -> Self {
        Self {
            chain_number,
            voter_parent_hash,
            proposer_block_votes,
        }
    }
}

/// Hashing the contents in a Merkle tree
impl Hashable for Content {
    fn hash(&self) -> H256 {
        let merkle_tree = MerkleTree::new(&self.proposer_block_votes);
        // TODO: Add chain number and voter_parent_hash in the hash
        return merkle_tree.root();
    }
}

pub fn genesis(chain_num: u16) -> Block {
    let all_zero: [u8; 32] = [0; 32];
    let content = Content {
        chain_number: chain_num,
        voter_parent_hash: (&all_zero).into(),
        proposer_block_votes: vec![],
    };
    return Block::new(
        (&all_zero).into(),
        0,
        0,
        (&all_zero).into(),
        vec![],
        BlockContent::Voter(content),
        all_zero.clone(),
        *DEFAULT_DIFFICULTY,
    );
}
