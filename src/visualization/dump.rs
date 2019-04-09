use crate::blockchain::edge::Edge as EdgeType;
use crate::blockchain::proposer::Status as ProposerStatus;
use crate::blockchain::voter::NodeStatus as VoterStatus;
use crate::blockchain::BlockChain;
use crate::crypto::hash::H256;
use std::collections::HashMap;

/// Struct to hold blockchain data to be dumped
#[derive(Serialize)]
pub struct Dump {
    pub edges: Vec<Edge>,
    pub proposer_levels: Vec<Vec<String>>,
    pub proposer_leaders: HashMap<u32, String>,
    pub voter_longest: Vec<String>,
    pub transaction_unconfirmed: Vec<String>,
    pub transaction_ordered: Vec<String>,
    pub transaction_unreferred: Vec<String>,
    pub proposer_nodes: HashMap<String, Proposer>,
    pub voter_nodes: HashMap<String, Voter>,
}

#[derive(Serialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edgetype: EdgeType,
}

#[derive(Serialize)]
pub struct Proposer {
    pub level: u32,
    pub status: ProposerStatus,
    pub votes: u16,
}

#[derive(Serialize)]
pub struct Voter {
    pub chain: u16,
    pub level: u32,
    pub status: VoterStatus,
}

pub fn dump_blockchain(chain: &BlockChain) -> String {
    let edges: Vec<Edge> = chain
        .graph
        .all_edges()
        .map(|e| Edge {
            from: e.0.into(),
            to: e.1.into(),
            edgetype: e.2.to_owned(),
        })
        .collect();
    let proposer_levels = chain
        .proposer_tree
        .prop_nodes
        .to_owned()
        .iter()
        .map(|v| v.iter().map(|h| h.into()).collect())
        .collect();
    let proposer_leaders = chain
        .proposer_tree
        .leader_nodes
        .iter()
        .map(|(l, h)| (l.to_owned(), h.into()))
        .collect();
    let voter_chain_best_blocks: Vec<String> = chain
        .voter_chains
        .iter()
        .map(|c| c.best_block.into())
        .collect();
    let transaction_unconfirmed: Vec<String> = chain
        .tx_blk_pool
        .not_in_ledger
        .iter()
        .map(|b| b.into())
        .collect();
    let transaction_ordered = chain
        .tx_blk_pool
        .ledger
        .to_owned()
        .iter()
        .map(|v| v.into())
        .collect();
    let transaction_unreferred: Vec<String> = chain
        .tx_blk_pool
        .unreferred
        .iter()
        .map(|b| b.into())
        .collect();
    let proposer_nodes: HashMap<String, Proposer> = chain
        .proposer_node_data
        .iter()
        .map(|(h, n)| {
            (
                h.into(),
                Proposer {
                    level: n.level,
                    status: n.leadership_status.to_owned(),
                    votes: n.votes,
                },
            )
        })
        .collect();
    let voter_nodes: HashMap<String, Voter> = chain
        .voter_node_data
        .iter()
        .map(|(h, n)| {
            (
                h.into(),
                Voter {
                    chain: n.chain_number,
                    level: n.level,
                    status: n.status.to_owned(),
                },
            )
        })
        .collect();
    let dump = Dump {
        edges: edges,
        proposer_levels: proposer_levels,
        proposer_leaders: proposer_leaders,
        voter_longest: voter_chain_best_blocks,
        transaction_unconfirmed: transaction_unconfirmed,
        transaction_ordered: transaction_ordered,
        transaction_unreferred: transaction_unreferred,
        proposer_nodes: proposer_nodes,
        voter_nodes: voter_nodes,
    };
    return serde_json::to_string_pretty(&dump).unwrap();
}
