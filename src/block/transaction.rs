use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::experiment::performance_counter::PayloadSize;
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
}

impl PayloadSize for Content {
    fn size(&self) -> usize {
        let mut total = 0;
        for t in &self.transactions {
            total += t.size();
        }
        total
    }
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        // TODO: we are hashing txs in a merkle tree.
        let merkle_tree = MerkleTree::new(&self.transactions);
        merkle_tree.root()
    }
}

#[cfg(test)]
pub mod tests {}
