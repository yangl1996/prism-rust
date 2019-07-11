use crate::block::{Block, Content};
use crate::crypto::hash::{Hashable, H256};

use std::convert::From;
use std::thread;
use std::collections::HashSet;
use log::{debug, warn};

use url::Url;
use tungstenite::{Message, connect};

#[derive(Serialize)]
pub struct ProposerBlock {
    /// Hash of this block
    id: String,
    /// Proposer parent
    parent: String,
    /// Miner id
    miner: String,
    /// Transaction refs
    transaction_refs: Vec<String>,
    /// Proposer refs
    proposer_refs: Vec<String>,
}
#[derive(Serialize)]
pub struct VoterBlock {
    /// Hash of this block
    id: String,
    /// Proposer parent
    parent: String,
    /// Miner id
    miner: String,
    /// Voting chain number
    chain: u16,
    /// Voter parent
    voter_parent: String,
    /// Votes
    votes: Vec<String>,
}
#[derive(Serialize)]
pub struct TransactionBlock {
    /// Hash of this block
    id: String,
    /// Proposer parent
    parent: String,
    /// Miner id
    miner: String,
}
#[derive(Serialize)]
pub struct UpdatedLedger {
    /// Hash of proposer blocks that are added to ledger 
    added: Vec<String>,
    /// Hash of proposer blocks that are removed from ledger 
    removed: Vec<String>,
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
        let ip =  {
            let mut bytes: [u8;4] = [0;4];
            bytes.copy_from_slice(&block.header.extra_content[26..30]);
            bytes
        };
        let port = {
            let mut bytes: [u8;2] = [0;2];
            bytes.copy_from_slice(&block.header.extra_content[30..32]);
            u16::from_be_bytes(bytes)
        };
        let miner = format!("{}.{}.{}.{}:{}",ip[0],ip[1],ip[2],ip[3],port);
        match &block.content {
            Content::Proposer(content) => {
                let b = ProposerBlock { id: hash.to_string(), parent: parent.to_string(), miner, transaction_refs: content.transaction_refs.iter().map(|x|x.to_string()).collect(), proposer_refs: content.proposer_refs.iter().map(|x|x.to_string()).collect()};
                DemoMsg::ProposerBlock(b)
            }
            Content::Voter(content) => {
                let b = VoterBlock { id: hash.to_string(), parent: parent.to_string(), miner, chain: content.chain_number, voter_parent: content.voter_parent.to_string(), votes: content.votes.iter().map(|x|x.to_string()).collect()};
                DemoMsg::VoterBlock(b)
            }
            Content::Transaction(_) => {
                let b = TransactionBlock { id: hash.to_string(), parent: parent.to_string(), miner };
                DemoMsg::TransactionBlock(b)
            }
        }
    }
}

pub fn new(url: &str, transaction_ratio: u32, voter_max: u16) -> crossbeam::Sender<DemoMsg> {
    let (sender, receiver) = crossbeam::channel::unbounded::<DemoMsg>();
    let url = url.to_owned();
    thread::spawn(move|| {
        if let Ok(parsed) = Url::parse(url.as_str()) {
            if let Ok((mut socket, _response)) = connect(parsed) {
                let mut transaction_cnt: u32 = 0;
                let mut transaction_set: HashSet<String> = HashSet::new();
                for mut msg in receiver.iter() {
                    match &mut msg {
                        DemoMsg::ProposerBlock(block) => {
                            block.transaction_refs.retain(|x|transaction_set.contains(x));
                        }
                        DemoMsg::VoterBlock(block) => {
                            if block.chain >= voter_max {
                                continue;
                            }
                        }
                        DemoMsg::TransactionBlock(block) => {
                            transaction_cnt +=1 ;
                            if transaction_cnt == transaction_ratio {
                                transaction_cnt = 0;
                                transaction_set.insert(block.id.clone());
                            } else {
                                continue;
                            }
                        }
                        _ => {}
                    }
                    match socket.write_message(Message::Text(serde_json::to_string_pretty(&msg).unwrap())) {
                        Ok(_) => (),
                        Err(e) => {
                            warn!("{}, drop demo msg receiver", e);
                            break;
                        }
                    };
                }
            } else {
                warn!("Fail to connect to websocket {}.", url);
            }
        } else {
            warn!("Fail to parse '{}'.", url);
        }
    });
    sender
}

pub fn insert_block_msg(block: &Block) -> DemoMsg {
    let msg: DemoMsg = block.into();
    msg
}

pub fn update_ledger_msg(added: &[H256], removed: &[H256]) -> DemoMsg {
    let added = added.iter().map(|x|x.to_string()).collect();
    let removed = removed.iter().map(|x|x.to_string()).collect();
    let msg: DemoMsg = DemoMsg::UpdatedLedger(UpdatedLedger{added, removed});
    msg
}

