pub mod header;
pub mod validator;
mod transaction;
mod proposer;
mod voter;

use crate::crypto::hash::{Hashable, H256};

/// A block in the Prism blockchain.
#[derive(Serialize, Deserialize, Debug, Hash)]
// TODO: discuss. I removed Default trait. It seems that the only place that it will be needed is in tests, to
// quickly generate some fake blocks. PartialEq is also removed for now
pub struct Block {
    /// The header of the block.
    header: header::Header,
    /// The content of the block. It could contain transactions, references, or votes, depending on
    /// the block type.
    content: Content,
    /// The sortition proof of the block.
    proof: Vec<H256>,
}

impl Block {
    // TODO: discuss. Sortition is removed, as it seems to be the job of the miner/validator. If it is needed, we
    // can easily add it. Also, we may not want to decide block type inside new(). It should
    // have been known when calling new(), since we are supplying the content. Miner needs to
    // decide block type, but it should reside within miner logic.
    
    /// Create a new block.
    pub fn new(parent: H256, timestamp: u64, nonce: u32, content_root: H256, sortition_proof: Vec<H256>, content: Content, extra_content: Vec<u32>, difficulty: u64) -> Self {
        let header = header::Header::new(parent, timestamp, nonce, content_root, extra_content, difficulty);
        Self {
            header: header,
            content: content,
            proof: sortition_proof,
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!();
    }
}

/// The content of a block. It could contain transactions, references, or votes, depending on the
/// type of the block.
#[derive(Serialize, Deserialize, Debug, Hash)]
pub enum Content {
    Transaction(transaction::Content),
    Proposer(proposer::Content),
    Voter(voter::Content),
}

