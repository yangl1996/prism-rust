use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};

#[derive(Serialize, Deserialize, Hash, Debug, Default, Clone)]
pub struct Content {
    /// List of transaction blocks referred by this proposer block
    pub transaction_block_hashes: Vec<H256>,
    /// List of proposer blocks referred by this proposer block
    pub proposer_block_hashes: Vec<H256>
    // todo(V): Add a coinbase transaction
    // todo(V): Might have to reference voter blocks to include their coinbase transactions .
}


impl Content{
    pub fn new(transaction_block_refs: Vec<H256>, proposer_block_refs : Vec<H256>) ->Self {
        Self{ transaction_block_hashes: transaction_block_refs, proposer_block_hashes: proposer_block_refs }
    }
}

/// Hashing the contents in a Merkle tree
impl Hashable for Content{
    fn hash(&self) -> H256 {
        /// todo(V): Add the proposer_block_refs too.
        let tx_merkle_tree = MerkleTree::new(&self.transaction_block_hashes);
        let prop_merkle_tree = MerkleTree::new(&self.proposer_block_hashes);
        return (*tx_merkle_tree.root()).clone();
    }
}

