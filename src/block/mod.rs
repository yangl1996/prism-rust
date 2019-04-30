pub mod header;
pub mod proposer;
pub mod transaction;
pub mod voter;
use crate::crypto::hash::{Hashable, H256};

/// A block in the Prism blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    /// The header of the block.
    pub header: header::Header,
    /// The content of the block. It could contain transactions, references, or votes, depending on
    /// the block type.
    pub content: Content,
    /// The sortition proof of the content. In addition to the content root in block header, we are
    /// able to verify that the block is mined on a set of content candidates.
    pub sortition_proof: Vec<H256>,
}

impl Block {
    /// Create a new block.
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

    // TODO: use another name
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
        // TODO: we are only hashing the header here.
        return self.header.hash();
    }
}

/// The content of a block. It could be transaction content, proposer content, or voter content,
/// depending on the type of the block.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Content {
    /// Transaction block content.
    Transaction(transaction::Content),
    /// Proposer block content.
    Proposer(proposer::Content),
    /// Voter block content.
    Voter(voter::Content),
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        match self {
            Content::Transaction(c) => c.hash(),
            Content::Proposer(c) => c.hash(),
            Content::Voter(c) => c.hash(),
        }
    }
}

#[cfg(test)]
mod tests {
//    use super::test_util;
    use crate::crypto::hash::Hashable;
    use rand::Rng;

    /*
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
    */

}
