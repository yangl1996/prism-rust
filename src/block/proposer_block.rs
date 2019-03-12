use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};

#[derive(Serialize, Deserialize, Hash, Debug, Default, Clone)]
pub struct Content {
    /// List of transaction blocks referred by this proposer block
    pub transaction_block_refs : Vec<H256>,
    /// List of proposer blocks referred by this proposer block
    pub proposer_block_refs : Vec<H256>
    // todo(V): No coinbase transactions added
    // todo(V): Might have to reference voter blocks.
}


impl Content{
    pub fn new(transaction_block_refs: Vec<H256>, proposer_block_refs : Vec<H256>) ->Self {
        Self{transaction_block_refs, proposer_block_refs}
    }
}

/// Hashing the contents in a Merkle tree
impl Hashable for Content{
    fn hash(&self) -> H256 {
        /// todo(V): Add the proposer_block_refs too.
        let merkle_tree = MerkleTree::new(&self.transaction_block_refs);
        return *merkle_tree.root();
    }
}

