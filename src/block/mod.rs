pub mod header;
pub mod pow_miner;
pub mod validator;
mod transaction;
mod proposer;
mod voter;

use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Transaction};

/// A block in the Prism blockchain.
#[derive(Serialize, Deserialize, Debug, Hash)]
// TODO: discuss. I removed Default trait. It seems that the only place that it will be needed is in tests, to
// quickly generate some fake blocks. PartialEq is also removed for now
pub struct Block {
    /// The header of the block.
    pub header: header::Header,
    /// The content of the block. It could contain transactions, references, or votes, depending on
    /// the block type.
    pub content: Content,
    /// The sortition proof of the block.
    pub sortition_proof: Vec<H256>,
}

impl Block {
    // TODO: discuss. Sortition is removed, as it seems to be the job of the miner/validator. If it is needed, we
    // can easily add it. Also, we may not want to decide block type inside new(). It should
    // have been known when calling new(), since we are supplying the content. Miner needs to
    // decide block type, but it should reside within miner logic.

    /// Create a new block from scratch.
    pub fn new(parent: H256, timestamp: u64, nonce: u32, content_root: H256, sortition_proof: Vec<H256>,
               content: Content, extra_content: Vec<u32>, difficulty: [u8; 32]) -> Self {
        let header = header::Header::new(parent, timestamp, nonce, content_root, extra_content, difficulty);
        Self {
            header: header,
            content: content,
            sortition_proof: sortition_proof,
        }
    }

    /// Create a new block from header.
    pub fn from_header(header: header::Header, sortition_proof: Vec<H256>, content: Content) -> Self {
        Self {
            header: header,
            content: content,
            sortition_proof: sortition_proof,
        }
    }



}

impl Hashable for Block {
    fn hash(&self)  -> H256 {
        unimplemented!();
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!();
    }
}

/// The content of a block. It could contain transactions, references, or votes, depending on the
/// type of the block.
#[derive(Serialize, Deserialize, Debug, Hash, Clone)]
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