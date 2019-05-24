use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::transaction::Transaction;

/// The content of a transaction block.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    pub transactions: Vec<Transaction>, // TODO: No coinbase transaction for now
}

impl Content {
    /// Create new transaction block content.
    pub fn new(transactions: Vec<Transaction>) -> Self {
        Self { transactions }
    }

    /// Return the size in bytes
    pub fn get_bytes(&self) -> u32 {
        let mut total_bytes = 0;
        for tx in self.transactions.iter() {
            total_bytes += tx.get_bytes();
        }
        return total_bytes;
    }
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        // TODO: we are hashing txs in a merkle tree.
        let merkle_tree = MerkleTree::new(&self.transactions);
        return merkle_tree.root();
    }
}
