use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};

use crate::transaction::{Transaction};

#[derive(Serialize, Deserialize, Hash, Debug, Default, Clone)]
pub struct Content {
    pub transactions: Vec<Transaction>
    // todo(V): No coinbase transaction added
}

impl Content{
    pub fn new(transactions: Vec<Transaction>) ->Self {
        Self{transactions}
    }
}

/// Hashing the contents in a Merkle tree
impl Hashable for Content{
    fn hash(&self) -> H256 {
        let merkle_tree = MerkleTree::new(&self.transactions);
        return (*merkle_tree.root()).clone();
    }
}


