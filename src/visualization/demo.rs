use crate::block::{Block, Content};
use crate::crypto::hash::{Hashable, H256};

use std::convert::From;
use std::thread;
use std::time::Duration;
use log::{debug, warn};

use std::sync::mpsc;
use url::Url;
use tungstenite::{Message, connect};

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

pub fn new(url: &str) -> crossbeam::Sender<String> {
    let (sender, receiver) = crossbeam::channel::unbounded::<String>();
    let url = url.to_owned();
    let mut msg_buffer: Option<String> = None;
    thread::spawn(move|| {
        let parsed = match Url::parse(url.as_str()) {
            Ok(x) => x,
            Err(e) => {
                warn!("Fail to parse '{}' due to {}.", url, e);
                loop { thread::park();}
            },
        };
        loop {
            let (mut socket, _response) = match connect(parsed.clone()) {
                Ok(x) => x,
                Err(e) => {
                    warn!("{}", e);
                    debug!("Retry connecting to websocket {} in 1000 ms.", url);
                    thread::sleep(Duration::from_millis(1000));
                    continue;
                }
            };
            if let Some(msg) = &msg_buffer {
                match socket.write_message(Message::Text(msg.clone())) {
                    Ok(_) => msg_buffer = None,
                    Err(e) => {
                        warn!("{}", e);
                        debug!("Retry connecting to websocket {} in 1000 ms.", url);
                        thread::sleep(Duration::from_millis(1000));
                        continue;
                    }
                };
            }
            for msg in receiver.iter() {
                match socket.write_message(Message::Text(msg.clone())) {
                    Ok(_) => (),
                    Err(e) => {
                        msg_buffer = Some(msg);
                        warn!("{}", e);
                        debug!("Retry connecting to websocket {} in 1000 ms.", url);
                        thread::sleep(Duration::from_millis(1000));
                        break;
                    }
                };
            }
        }
    });
    sender
}

pub fn insert_block_msg(block: &Block) -> String {
    let msg: DemoMsg = block.into();
    let json: String = serde_json::to_string_pretty(&msg).unwrap();
    json
}

pub fn update_ledger_msg(added: &[H256], removed: &[H256]) -> String {
    if added.is_empty() && removed.is_empty() {
        return String::from("");
    }
    let added = added.iter().map(|x|x.to_string()).collect();
    let removed = removed.iter().map(|x|x.to_string()).collect();
    let msg: DemoMsg = DemoMsg::UpdatedLedger(UpdatedLedger{added, removed});
    let json: String = serde_json::to_string_pretty(&msg).unwrap();
    json
}

