use super::Block;
use super::Content as BlockContent;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;

/// The content of a voter block.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    /// ID of the voter chain this block is attaching to.
    pub chain_number: u16,
    /// Hash of the parent voter block.
    pub voter_parent: H256,
    /// List of votes on proposer blocks.
    pub votes: Vec<H256>,
}

impl Content {
    /// Create new voter block content.
    pub fn new(chain_number: u16, voter_parent: H256, votes: Vec<H256>) -> Self {
        Self {
            chain_number,
            voter_parent,
            votes,
        }
    }

    /// Return the size in bytes
    pub fn get_bytes(&self) -> u32 {
        return (2+32+self.votes.len()*32) as u32;
    }
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        // TODO: we are hashing in a merkle tree. why do we need so?
        let merkle_tree = MerkleTree::new(&self.votes);
        // TODO: Add chain number and voter_parent_hash in the hash
        return merkle_tree.root();
    }
}

/// Generate the genesis block of the voter chain with the given chain ID.
pub fn genesis(chain_num: u16) -> Block {
    let all_zero: [u8; 32] = [0; 32];
    let content = Content {
        chain_number: chain_num,
        voter_parent: VOTER_GENESIS_HASHES[chain_num as usize],
        votes: vec![],
    };
    // TODO: this block will definitely not pass validation. We depend on the fact that genesis
    // blocks are added to the system at initialization. Seems like a moderate hack.
    return Block::new(
        all_zero.into(),
        0,
        0,
        all_zero.into(),
        vec![],
        BlockContent::Voter(content),
        all_zero,
        *DEFAULT_DIFFICULTY,
    );
}
