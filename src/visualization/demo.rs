use crate::block::{Block, Content};
use crate::crypto::hash::{Hashable, H256};

use std::convert::From;
use std::fs::File;
use std::sync::Mutex;
use std::io::prelude::*;

pub struct Server {
    handle: Mutex<File>
}

#[derive(Serialize)]
pub struct ProposerBlock {
    /// Hash of this block
    pub id: String,
    /// Proposer parent
    pub parent: String,
    /// Transaction refs
    pub transaction_refs: Vec<String>,
    /// Proposer refs
    pub proposer_refs: Vec<String>,
}
#[derive(Serialize)]
pub struct VoterBlock {
    /// Hash of this block
    pub id: String,
    /// Proposer parent
    pub parent: String,
    /// Voting chain number
    pub chain: u16,
}
#[derive(Serialize)]
pub struct TransactionBlock {
    /// Hash of this block
    pub id: String,
    /// Proposer parent
    pub parent: String,
}
#[derive(Serialize)]
pub enum DemoBlock {
    ProposerBlock(ProposerBlock),
    VoterBlock(VoterBlock),
    TransactionBlock(TransactionBlock),
}

impl From<&Block> for String {
    fn from(block: &Block) -> Self {
        let hash = block.hash();
        let parent = block.header.parent;
        match &block.content {
            Content::Proposer(content) => {
                let b = ProposerBlock { id: hash.to_string(), parent: parent.to_string(), transaction_refs: content.transaction_refs.iter().map(|x|x.to_string()).collect(), proposer_refs: content.proposer_refs.iter().map(|x|x.to_string()).collect()};
                serde_json::to_string_pretty(&DemoBlock::ProposerBlock(b)).unwrap()
            }
            Content::Voter(content) => {
                let b = VoterBlock { id: hash.to_string(), parent: parent.to_string(), chain: content.chain_number };
                serde_json::to_string_pretty(&DemoBlock::VoterBlock(b)).unwrap()
            }
            Content::Transaction(_) => {
                let b = TransactionBlock { id: hash.to_string(), parent: parent.to_string() };
                serde_json::to_string_pretty(&DemoBlock::TransactionBlock(b)).unwrap()
}
        }
    }
}

impl Server {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self { handle: Mutex::new(file) })
    }

    pub fn send(&self, block: &Block) -> std::io::Result<()> {
        let json: String = block.into();
        let mut handle = self.handle.lock().unwrap();
        writeln!(handle, "{}", json).unwrap();
        Ok(())
    }

    /*
    pub fn print(&self) {
        let handle = self.handle.lock().unwrap();
        println!("{:?}", handle);
    }
    */
}
