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
    metadata.random_source = (&VOTER_GENESIS_HASHES[chain_num as usize]).into();
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

#[cfg(test)]
pub mod test {
    use super::super::proposer::tests::*;
    use super::super::transaction::tests::*;
    use super::super::voter::Content as voter_Content; // TODO: name change to VoterContent
    use super::super::{Block, Content};
    use crate::crypto::hash::{Hashable, H256};

    #[test]
    fn test_hash() {
        let block = sample_voter_content();
        let block_hash_should_be = sample_voter_content1_hash_should_be();
        assert_eq!(block.hash(), block_hash_should_be);
    }

    // Voter block stuff
    pub fn sample_voter_content() -> voter_Content {
        let chain_number = 0;
        let voter_parent_hash =
            (&hex!("0000000100000001000000010000000100000001000000010000000100000001")).into();
        let proposer_block1 = sample_proposer_block1();
        let proposer_block2 = sample_proposer_block2();
        let proposer_block_votes = vec![proposer_block1.hash(), proposer_block2.hash()];
        return voter_Content::new(chain_number, voter_parent_hash, proposer_block_votes);
    }
    pub fn sample_voter_content1_hash_should_be() -> H256 {
        let transaction_content_hash: H256 =
            (&hex!("72f9ab129de520ea674ef0b3b7ded5144ef76cb11d133a2bfa21b15057ed84d3")).into();
        return transaction_content_hash;
    }
}
