pub mod generator;
pub mod header;
pub mod proposer;
mod test_util;
pub mod transaction;
pub mod voter;

use crate::config::*;
use crate::crypto::hash::{Hashable, H256};

/// A block in the Prism blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
// TODO: discuss. I removed Default trait. It seems that the only place that it will be needed is in tests, to
// quickly generate some fake blocks. PartialEq is also removed for now
pub struct Block {
    /// The header of the block.
    pub header: header::Header,
    /// The content of the block. It could contain transactions, references, or votes, depending on
    /// the block type.
    pub content: Content,
    /// The sortition proof of the content.  This 'connects' the content to the content root in the header.
    pub sortition_proof: Vec<H256>,
}

impl Block {
    /// Create a new block from scratch.
    pub fn new(
        parent: H256,
        timestamp: u64,
        nonce: u32,
        content_root: H256,
        sortition_proof: Vec<H256>,
        content: Content,
        extra_content: [u8; 32],
        difficulty: H256,
    ) -> Self {
        let header = header::Header::new(
            parent,
            timestamp,
            nonce,
            content_root,
            extra_content,
            difficulty,
        );
        Self {
            header,
            content,
            sortition_proof,
        }
    }

    /// Create a new block from header.
    pub fn from_header(
        header: header::Header,
        content: Content,
        sortition_proof: Vec<H256>,
    ) -> Self {
        Self {
            header,
            content,
            sortition_proof,
        }
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        // hash of the header seals the block content and the sortition proof
        return self.header.hash();
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "hash: {}; type: {}", self.header.hash(), self.content)?; // Ignoring status for now
        Ok(())
    }
}

/// The content of a block. It could contain transactions, references, or votes, depending on the
/// type of the block.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Content {
    Transaction(transaction::Content),
    Proposer(proposer::Content),
    Voter(voter::Content),
}

// todo: This is a bad coding.
impl Hashable for Content {
    fn hash(&self) -> H256 {
        match self {
            Content::Transaction(c) => c.hash(),
            Content::Proposer(c) => c.hash(),
            Content::Voter(c) => c.hash(),
        }
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Content::Transaction(_c) => {
                write!(f, "Transaction block")?;
                Ok(())
            }
            Content::Proposer(_c) => {
                write!(f, "Proposer block")?;
                Ok(())
            }
            Content::Voter(c) => {
                write!(f, "Voter block @ ({})", c.chain_number)?;
                Ok(())
            }
        }
    }
}
impl Default for Content {
    fn default() -> Self {
        Content::Transaction(transaction::Content::default())
    }
}

#[cfg(test)]
mod tests {
    use super::test_util;
    use crate::crypto::hash::Hashable;
    use rand::Rng;

    #[test]
    #[ignore]
    fn check_transaction_content_hash() {
        let transaction_content = test_util::sample_transaction_content();
        let transaction_content_hash_shouldbe =
            test_util::sample_transaction_content_hash_shouldbe();
        assert_eq!(
            transaction_content.hash(),
            transaction_content_hash_shouldbe
        );
    }

    #[test]
    fn check_proposer_content_hash() {
        let proposer_content = test_util::sample_proposer_content1();
        let proposer_content_hash_shouldbe = test_util::sample_proposer_content1_hash_shouldbe();
        assert_eq!(proposer_content.hash(), proposer_content_hash_shouldbe);
    }

    #[test]
    fn check_voter_content_hash() {
        let voter_content = test_util::sample_voter_content();
        let voter_content_hash_shouldbe = test_util::sample_voter_content1_hash_shouldbe();
        assert_eq!(voter_content.hash(), voter_content_hash_shouldbe);
    }

    #[test]
    fn check_block_hash() {
        let transaction_block = test_util::sample_transaction_block(); // Arbitrarily choosing tx block
        let transaction_block_hash_shouldbe = test_util::sample_header_hash_should_be();
        assert_eq!(transaction_block.hash(), transaction_block_hash_shouldbe);
    }

    #[test]
    fn block_sortition_proof() {
        let mut rng = rand::thread_rng();
        let _index = rng.gen_range(0, 101);
        let _block = test_util::sample_mined_block(5);
        // todo: Verify the sortition proof.
    }

}
