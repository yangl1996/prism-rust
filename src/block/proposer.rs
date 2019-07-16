use super::Block;
use super::Content as BlockContent;
use super::pos_metadata::Metadata;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::experiment::performance_counter::PayloadSize;

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
}

impl PayloadSize for Content {
    fn size(&self) -> usize {
        return std::mem::size_of::<H256>()
            * (self.transaction_refs.len() + self.proposer_refs.len());
    }
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        let tx_merkle_tree = MerkleTree::new(&self.transaction_refs);
        let prop_merkle_tree = MerkleTree::new(&self.proposer_refs);
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(tx_merkle_tree.root().as_ref());
        bytes[32..64].copy_from_slice(prop_merkle_tree.root().as_ref());
        return ring::digest::digest(&ring::digest::SHA256, &bytes).into();
    }
}

/// Generate the genesis block of the proposer chain.
pub fn genesis() -> Block {
    let content = Content {
        transaction_refs: vec![],
        proposer_refs: vec![],
    };
    let mut metadata = Metadata::default();
    // TODO metadata.timestamp
    metadata.random_source = PROPOSER_GENESIS_RAND.clone();
    let all_zero: [u8; 32] = [0; 32];
    // TODO: this will not pass validation.
    return Block::new(
        all_zero.into(),
        metadata,
        H256::default(),
        all_zero,
        *DEFAULT_DIFFICULTY,
        vec![],
        BlockContent::Proposer(content),
    );
}

#[cfg(test)]
pub mod tests {
    use super::super::header::tests::*;
    use super::super::proposer::Content as ProposerContent;
    use super::super::transaction::tests::*;

    use super::super::{Block, Content};
    use crate::crypto::hash::{Hashable, H256};
    use crate::crypto::merkle::MerkleTree;
    use crate::transaction::{Authorization, CoinId, Input, Output, Transaction};
    use std::cell::RefCell;

    #[test]
    fn test_hash() {
        let block = sample_proposer_block1();
        let block_hash_should_be = sample_proposer_block1_hash_should_be();
        assert_eq!(block.hash(), block_hash_should_be);
    }

    #[test]
    fn test_hash2() {
        let block = sample_proposer_block2();
        let block_hash_should_be = sample_proposer_block2_hash_should_be();
        assert_eq!(block.hash(), block_hash_should_be);
    }
    macro_rules! gen_hashed_data {
        () => {{
            vec![
                (&hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into(),
                (&hex!("0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b")).into(),
                (&hex!("0403020108070605040302010807060504030201080706050403020108070605")).into(),
                (&hex!("1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a")).into(),
                (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into(),
                (&hex!("0000000100000001000000010000000100000001000000010000000100000001")).into(),
            ]
        }};
    }

    // Proposer block stuffs
    /// Returns sample content of a proposer block containing only tx block hashes
    pub fn sample_proposer_content1() -> ProposerContent {
        let tx_block = sample_transaction_block();
        let transaction_refs = vec![tx_block.hash()];

        let proposer_block_content = ProposerContent {
            transaction_refs,
            proposer_refs: vec![],
        };
        return proposer_block_content;
    }

    /// Returns sample a proposer block 1 and 2
    pub fn sample_proposer_block1() -> Block {
        let proposer_block_content = sample_proposer_content1();
        let mut header = sample_header(); // The content root is incorrect
        header.content_merkle_root = proposer_block_content.hash();
        let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
        return Block {
            header,
            content: Content::Proposer(proposer_block_content),
            sortition_proof,
        };
    }

    pub fn sample_proposer_block1_hash_should_be() -> H256 {
        let transaction_content_hash: H256 =
            (&hex!("df3934c428cc3d5de486f28b3e7becaecec17d43d2c40d50fb20c7ff2fd3be1f")).into();
        return transaction_content_hash;
    }

    /// Returns sample content of a proposer block containing only tx block hashes and prop block hashes
    pub fn sample_proposer_content2() -> ProposerContent {
        let tx_block = sample_transaction_block();
        let transaction_refs = vec![tx_block.hash()];
        let propose_block = sample_proposer_block1();
        let proposer_refs = vec![propose_block.hash()];
        let proposer_block_content = ProposerContent {
            transaction_refs,
            proposer_refs,
        };
        return proposer_block_content;
    }
    pub fn sample_proposer_block2() -> Block {
        let proposer_block_content = sample_proposer_content2();
        let mut header = sample_header(); // The content root is incorrect
        header.content_merkle_root = proposer_block_content.hash();
        let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
        return Block {
            header,
            content: Content::Proposer(proposer_block_content),
            sortition_proof,
        };
    }
    pub fn sample_proposer_block2_hash_should_be() -> H256 {
        let transaction_content_hash: H256 =
            (&hex!("8298338734bbea798e1577c32fe536335fbdf2e9629a2044bbe4694339480bb2")).into();
        return transaction_content_hash;
    }

}
