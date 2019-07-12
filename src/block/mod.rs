pub mod header;
pub mod proof;
pub mod proposer;
pub mod transaction;
pub mod voter;

use crate::crypto::hash::{Hashable, H256};
use crate::experiment::performance_counter::PayloadSize;

/// A block in the Prism blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    /// The header of the block.
    pub header: header::Header,
    /// The content of the block. It could contain transactions, references, or votes, depending on
    /// the block type.
    pub content: Content,
}

impl Block {
    /// Create a new block.
    pub fn new(
        parent: H256,
        parent_random_source: header::RandomSource,
        timestamp: u128,
        pos_proof: proof::Proof,
        random_source: header::RandomSource,
        content_root: H256,
        extra_content: [u8; 32],
        difficulty: H256,
        header_signature: Vec<u8>,
        content: Content,
    ) -> Self {
        let header = header::Header::new(
            parent,
            parent_random_source,
            timestamp,
            pos_proof,
            random_source,
            content_root,
            extra_content,
            difficulty,
            header_signature,
        );
        Self {
            header,
            content,
        }
    }

    // TODO: use another name
    /// Create a new block from header.
    pub fn from_header(
        header: header::Header,
        content: Content,
    ) -> Self {
        Self {
            header,
            content,
        }
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        // TODO: we are only hashing the header here.
        return self.header.hash();
    }
}

impl PayloadSize for Block {
    fn size(&self) -> usize {
        return std::mem::size_of::<header::Header>()
            + self.content.size()

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

impl PayloadSize for Content {
    fn size(&self) -> usize {
        // TODO: we are not counting the 2 bits that are used to store block type
        match self {
            Content::Transaction(c) => c.size(),
            Content::Proposer(c) => c.size(),
            Content::Voter(c) => c.size(),
        }
    }
}
//
//#[cfg(any(test, feature = "test-utilities"))]
//pub mod tests {
//
//    use super::*;
//    use crate::config;
//    use crate::transaction::Transaction;
//    use rand::Rng;
//
//    macro_rules! random_nonce {
//        () => {{
//            let mut rng = rand::thread_rng();
//            let random_u32: u32 = rng.gen();
//            random_u32
//        }};
//    }
//
//    pub fn proposer_block(
//        parent: H256,
//        timestamp: u128,
//        proposer_refs: Vec<H256>,
//        transaction_refs: Vec<H256>,
//    ) -> Block {
//        let content = Content::Proposer(proposer::Content {
//            transaction_refs,
//            proposer_refs,
//        });
//        let content_hash = content.hash();
//        Block::new(
//            parent,
//            timestamp,
//            random_nonce!(),
//            content_hash,
//            vec![content_hash],
//            content,
//            [0u8; 32],
//            *config::DEFAULT_DIFFICULTY,
//        )
//    }
//
//    pub fn voter_block(
//        parent: H256,
//        timestamp: u128,
//        chain_number: u16,
//        voter_parent: H256,
//        votes: Vec<H256>,
//    ) -> Block {
//        let content = Content::Voter(voter::Content {
//            chain_number,
//            voter_parent,
//            votes,
//        });
//        let content_hash = content.hash();
//        Block::new(
//            parent,
//            timestamp,
//            random_nonce!(),
//            content_hash,
//            vec![content_hash],
//            content,
//            [0u8; 32],
//            *config::DEFAULT_DIFFICULTY,
//        )
//    }
//
//    pub fn transaction_block(
//        parent: H256,
//        timestamp: u128,
//        transactions: Vec<Transaction>,
//    ) -> Block {
//        let content = Content::Transaction(transaction::Content { transactions });
//        let content_hash = content.hash();
//        Block::new(
//            parent,
//            timestamp,
//            random_nonce!(),
//            content_hash,
//            vec![content_hash],
//            content,
//            [0u8; 32],
//            *config::DEFAULT_DIFFICULTY,
//        )
//    }
//}
