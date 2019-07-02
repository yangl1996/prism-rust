use crate::block::{Block, Content};
use crate::crypto::hash::{Hashable, H256};

use std::convert::From;
use std::thread;

use std::sync::mpsc;
use websocket::client::ClientBuilder;
use websocket::client::sync::Client;
use websocket::stream::sync::TcpStream;
use websocket::message::OwnedMessage;

/*
pub fn new(url: String) -> Handle {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("demo websocket server".to_owned())
        .spawn(move || {
            connect(url, |out| {
                Context {
                    out,
                    chan: receiver
                }
            }).unwrap()
        })
        .unwrap();
    Handle { chan: sender }
}

pub struct Context {
    out: Sender,
    chan: mpsc::Receiver<String>,
}

#[derive(Clone)]
pub struct Handle {
    chan: mpsc::Sender<String>
}
*/
pub struct Server {
    handle: Client<TcpStream>
}

#[derive(Serialize)]
struct ProposerBlock {
    /// Hash of this block
    id: String,
    /// Proposer parent
    parent: String,
    /// Transaction refs
    transaction_refs: Vec<String>,
    /// Proposer refs
    proposer_refs: Vec<String>,
}
#[derive(Serialize)]
struct VoterBlock {
    /// Hash of this block
    id: String,
    /// Proposer parent
    parent: String,
    /// Voting chain number
    chain: u16,
    /// Voter parent
    voter_parent: String,
    /// Votes
    votes: Vec<String>,
}
#[derive(Serialize)]
struct TransactionBlock {
    /// Hash of this block
    id: String,
    /// Proposer parent
    parent: String,
}
#[derive(Serialize)]
struct UpdatedLedger {
    /// Hash of proposer blocks that are added to ledger 
    added: Vec<String>,
    /// Hash of proposer blocks that are removed from ledger 
    removed: Vec<String>,
}
#[derive(Serialize)]
enum DemoMsg {
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
    pub fn new(url: &str) -> Server {
        let client = ClientBuilder::new(url)
		.unwrap()
		.add_protocol("rust-websocket")
		.connect_insecure()
		.unwrap();
        Server {handle: client}
    }

    pub fn test(&mut self, s: &str) {
        self.handle.send_message(&OwnedMessage::Text(s.to_string())).unwrap();
        //self.chan.send(json).unwrap();
    }
    /*
    pub fn insert_block(&self, block: &Block) -> Result<()> {
        let msg: DemoMsg = block.into();
        let json: String = serde_json::to_string_pretty(&msg).unwrap();
        self.handle.send(json)
        //self.chan.send(json).unwrap();
    }

    pub fn update_ledger(&self, added: &[H256], removed: &[H256]) -> Result<()> {
        if added.is_empty() && removed.is_empty() {
            return Ok(());
        }
        let added = added.iter().map(|x|x.to_string()).collect();
        let removed = removed.iter().map(|x|x.to_string()).collect();
        let msg: DemoMsg = DemoMsg::UpdatedLedger(UpdatedLedger{added, removed});
        let json: String = serde_json::to_string_pretty(&msg).unwrap();
        self.handle.send(json)
    }
    */

}
/*
impl Handler for Context {
    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        //let chan_copy = self.chan.clone();
        for msg in self.chan.iter() {
            self.out.send(msg).unwrap();
        }
        Ok(())
    }

}
*/
