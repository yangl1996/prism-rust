use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use super::Block;
use super::Content as BlockContent;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    /// List of transaction blocks referred by this proposer block
    pub transaction_block_hashes: Vec<H256>,
    /// List of proposer blocks referred by this proposer block
    pub proposer_block_hashes: Vec<H256>, // todo(V): Add a coinbase transaction
                                          // todo(V): Might have to reference voter blocks to include their coinbase transactions .
}

impl Content {
    pub fn new(transaction_block_refs: Vec<H256>, proposer_block_refs: Vec<H256>) -> Self {
        Self {
            transaction_block_hashes: transaction_block_refs,
            proposer_block_hashes: proposer_block_refs,
        }
    }
}

/// Hashing the contents in a Merkle tree
impl Hashable for Content {
    fn hash(&self) -> H256 {
        // TODO(V): Add the proposer_block_refs too.
        let tx_merkle_tree = MerkleTree::new(&self.transaction_block_hashes);
        let _prop_merkle_tree = MerkleTree::new(&self.proposer_block_hashes);
        // TODO: why do we calculate prop_merkle_tree when we don't use it?
        return tx_merkle_tree.root();
    }
}

pub fn genesis() -> Block {
    let content = Content {
        transaction_block_hashes: vec![],
        proposer_block_hashes: vec![],
    };
    let all_zero: [u8; 32] = [0; 32];
    return Block::new(
        (&all_zero).into(),
        0,
        0,
        (&all_zero).into(),
        vec![],
        BlockContent::Proposer(content),
        all_zero.clone(),
        all_zero.clone(),
    );
}
