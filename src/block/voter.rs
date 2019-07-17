use super::Block;
use super::Content as BlockContent;
use super::pos_metadata::Metadata;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::experiment::performance_counter::PayloadSize;

/// The content of a voter block.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    /// ID of the voter chain this block is attaching to.
    pub chain_number: u16,
    /// List of votes on proposer blocks.
    pub votes: Vec<H256>,
}

impl Content {
    /// Create new voter block content.
    pub fn new(chain_number: u16, votes: Vec<H256>) -> Self {
        Self {
            chain_number,
            votes,
        }
    }
}

impl PayloadSize for Content {
    fn size(&self) -> usize {
        return std::mem::size_of::<u16>()
            + std::mem::size_of::<H256>()
            + self.votes.len() * std::mem::size_of::<H256>();
    }
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        // TODO: we are hashing in a merkle tree. why do we need so?
        let merkle_tree = MerkleTree::new(&self.votes);
        let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
        ctx.update(&self.chain_number.to_be_bytes());
        ctx.update(merkle_tree.root().as_ref());
        let digest = ctx.finish();
        digest.into()
    }
}

/// Generate the genesis block of the voter chain with the given chain ID.
pub fn genesis(chain_num: u16) -> Block {
    let all_zero: [u8; 32] = [0; 32];
    let mut metadata = Metadata::default();
    metadata.random_source = VOTER_GENESIS_RANDS[chain_num as usize].clone();
    let content = Content {
        chain_number: chain_num,
        votes: vec![],
    };
    // TODO: this block will definitely not pass validation. We depend on the fact that genesis
    // blocks are added to the system at initialization. Seems like a moderate hack.
    return Block::new(
        VOTER_GENESIS_HASHES[chain_num as usize],
        metadata,
        H256::default(),
        all_zero,
        *DEFAULT_DIFFICULTY,
        vec![],
        BlockContent::Voter(content),
    );
}
