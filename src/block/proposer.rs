use super::Block;
use super::Content as BlockContent;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;

/// The content of a proposer block.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    /// List of transaction blocks referred by this proposer block.
    pub transaction_refs: Vec<H256>,
    /// List of proposer blocks referred by this proposer block.
    pub proposer_refs: Vec<H256>,
    // TODO: coinbase transaction, and maybe refer to voter blocks to include their coinbase
    // transactions.
}

impl Content {
    /// Create new proposer block content.
    pub fn new(transaction_refs: Vec<H256>, proposer_refs: Vec<H256>) -> Self {
        Self {
            transaction_refs,
            proposer_refs,
        }
    }

    /// Return the size in bytes
    pub fn get_bytes(&self) -> u32 {
        return (self.transaction_refs.len() * 32 + self.proposer_refs.len() * 32) as u32;
    }
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        // TODO: include proposer merkle tree
        // TODO: why do we need a merkle tree here? simply hashing all the bytes is much faster and
        // more straightforward.
        let tx_merkle_tree = MerkleTree::new(&self.transaction_refs);
        let _prop_merkle_tree = MerkleTree::new(&self.proposer_refs);
        return tx_merkle_tree.root();
    }
}

/// Generate the genesis block of the proposer chain.
pub fn genesis() -> Block {
    let content = Content {
        transaction_refs: vec![],
        proposer_refs: vec![],
    };
    let all_zero: [u8; 32] = [0; 32];
    // TODO: this will not pass validation.
    return Block::new(
        all_zero.into(),
        0,
        0,
        all_zero.into(),
        vec![],
        BlockContent::Proposer(content),
        all_zero,
        *INITIAL_DIFFICULTY,
    );
}
