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
    /// Voter parent
    pub voter_parent: String,
    /// Votes
    pub votes: Vec<String>,
}
#[derive(Serialize)]
pub struct TransactionBlock {
    /// Hash of this block
    pub id: String,
    /// Proposer parent
    pub parent: String,
}
#[derive(Serialize)]
pub struct UpdatedLedger {
    /// Hash of proposer blocks that are added to ledger 
    pub added: Vec<String>,
    /// Hash of proposer blocks that are removed from ledger 
    pub removed: Vec<String>,
}
#[derive(Serialize)]
pub enum DemoMsg {
    ProposerBlock(ProposerBlock),
    VoterBlock(VoterBlock),
    TransactionBlock(TransactionBlock),
    UpdatedLedger(UpdatedLedger),
}

impl From<&Block> for DemoMsg {
    fn from(block: &Block) -> Self {
        let hash = block.hash();
        let parent = block.header.parent;
        match &block.content {
            Content::Proposer(content) => {
                let b = ProposerBlock { id: hash.to_string(), parent: parent.to_string(), transaction_refs: content.transaction_refs.iter().map(|x|x.to_string()).collect(), proposer_refs: content.proposer_refs.iter().map(|x|x.to_string()).collect()};
                DemoMsg::ProposerBlock(b)
            }
            Content::Voter(content) => {
                let b = VoterBlock { id: hash.to_string(), parent: parent.to_string(), chain: content.chain_number, voter_parent: content.voter_parent.to_string(), votes: content.votes.iter().map(|x|x.to_string()).collect()};
                DemoMsg::VoterBlock(b)
            }
            Content::Transaction(_) => {
                let b = TransactionBlock { id: hash.to_string(), parent: parent.to_string() };
                DemoMsg::TransactionBlock(b)
            }
        }
    }
}

impl Server {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self { handle: Mutex::new(file) })
    }

    pub fn insert_block(&self, block: &Block) -> std::io::Result<()> {
        let msg: DemoMsg = block.into();
        let json: String = serde_json::to_string_pretty(&msg).unwrap();
        let mut handle = self.handle.lock().unwrap();
        writeln!(handle, "{}", json)?;
        Ok(())
    }

    pub fn update_ledger(&self, added: &[H256], removed: &[H256]) -> std::io::Result<()> {
        if added.is_empty() && removed.is_empty() {
            return Ok(());
        }
        let added = added.iter().map(|x|x.to_string()).collect();
        let removed = removed.iter().map(|x|x.to_string()).collect();
        let msg: DemoMsg = DemoMsg::UpdatedLedger(UpdatedLedger{added, removed});
        let json: String = serde_json::to_string_pretty(&msg).unwrap();
        let mut handle = self.handle.lock().unwrap();
        writeln!(handle, "{}", json)?;
        Ok(())
    }

    /*
    pub fn print(&self) {
        let handle = self.handle.lock().unwrap();
        println!("{:?}", handle);
    }
    */
}
