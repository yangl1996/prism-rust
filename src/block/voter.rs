use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};

#[derive(Serialize, Deserialize, Hash, Debug, Default, Clone)]
pub struct Content {
    /// Voter chain id
    pub chain_id: u16,
    /// Hash of the parent voter block.
    pub voter_parent_hash: H256,
    /// List of proposer block votes.
    pub proposer_block_votes : Vec<H256>
}

impl Content{
    pub fn new(chain_id: u16, voter_parent_hash: H256, proposer_block_votes: Vec<H256>) ->Self {
        Self{chain_id, voter_parent_hash, proposer_block_votes}
    }
}

/// Hashing the contents in a Merkle tree
impl Hashable for Content{
    fn hash(&self) -> H256 {
        let merkle_tree = MerkleTree::new(&self.proposer_block_votes);
        return *merkle_tree.root();
    }
}