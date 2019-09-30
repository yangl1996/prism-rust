use crate::block;
use crate::block::proposer;
use crate::block::voter;
use crate::crypto::hash::H256;
use bigint::uint::U256;

const AVG_TX_SIZE: u32 = 168; // average size of a transaction (in Bytes)
const PROPOSER_TX_REF_HEADROOM: f32 = 10.0;
const SORTITION_PRECISION: u64 = std::u64::MAX;

// Chain IDs
pub const PROPOSER_INDEX: u16 = 0;
pub const TRANSACTION_INDEX: u16 = 1;
pub const FIRST_VOTER_INDEX: u16 = 2;

#[derive(Clone)]
pub struct BlockchainConfig {
    /// Number of voter chains.
    pub voter_chains: u16,
    /// Maximum size of a transaction block in terms of transactions.
    pub tx_txs: u32,
    /// Maximum number of transaction block references in a proposer block.
    pub proposer_tx_refs: u32,
    /// Proposer block minng rate in blocks/sec.
    pub proposer_mining_rate: f32,
    /// Voter block minng rate for one voter chain, in blocks/sec.
    pub voter_mining_rate: f32,
    /// Transaction block minng rate in blocks/sec.
    pub tx_mining_rate: f32,
    /// Hash of proposer genesis block.
    pub proposer_genesis: H256,
    /// Hashes of voter genesis blocks.
    pub voter_genesis: Vec<H256>,
    total_mining_rate: f32,
    total_sortition_width: U256,
    proposer_sortition_width: U256,
    voter_sortition_width: U256,
    tx_sortition_width: U256,
}

impl BlockchainConfig {
    pub fn new(
        voter_chains: u16,
        tx_size: u32,
        tx_throughput: u32,
        proposer_rate: f32,
        voter_rate: f32,
    ) -> Self {
        let tx_txs = tx_size / AVG_TX_SIZE;
        let proposer_genesis: H256 = {
            let mut raw_hash: [u8; 32] = [0; 32];
            let bytes = PROPOSER_INDEX.to_be_bytes();
            raw_hash[30] = bytes[0];
            raw_hash[31] = bytes[1];
            raw_hash.into()
        };
        let voter_genesis_hashes: Vec<H256> = {
            let mut v: Vec<H256> = vec![];
            for chain_num in 0..voter_chains {
                let mut raw_hash: [u8; 32] = [0; 32];
                let bytes = (chain_num + FIRST_VOTER_INDEX).to_be_bytes();
                raw_hash[30] = bytes[0];
                raw_hash[31] = bytes[1];
                v.push(raw_hash.into());
            }
            v
        };
        let tx_mining_rate: f32 = {
            let tx_thruput: f32 = tx_throughput as f32;
            let tx_txs: f32 = tx_txs as f32;
            tx_thruput / tx_txs
        };
        let total_mining_rate: f32 =
            proposer_rate + voter_rate * voter_chains as f32 + tx_mining_rate;
        let proposer_width: u64 = {
            let precise: f32 = (proposer_rate / total_mining_rate) * SORTITION_PRECISION as f32;
            precise.ceil() as u64
        };
        let voter_width: u64 = {
            let precise: f32 = (voter_rate / total_mining_rate) * SORTITION_PRECISION as f32;
            precise.ceil() as u64
        };
        let tx_width: u64 =
            SORTITION_PRECISION - proposer_width - voter_width * voter_chains as u64;
        return Self {
            voter_chains: voter_chains,
            tx_txs: tx_txs,
            proposer_tx_refs: (tx_mining_rate / proposer_rate * PROPOSER_TX_REF_HEADROOM).ceil()
                as u32,
            proposer_mining_rate: proposer_rate,
            voter_mining_rate: voter_rate,
            tx_mining_rate: tx_mining_rate,
            proposer_genesis: proposer_genesis,
            voter_genesis: voter_genesis_hashes,
            total_mining_rate: total_mining_rate,
            total_sortition_width: SORTITION_PRECISION.into(),
            proposer_sortition_width: proposer_width.into(),
            voter_sortition_width: voter_width.into(),
            tx_sortition_width: tx_width.into(),
        };
    }

    pub fn sortition_hash(&self, hash: &H256, difficulty: &H256) -> Option<u16> {
        let hash = U256::from_big_endian(hash.as_ref());
        let difficulty = U256::from_big_endian(difficulty.as_ref());
        let multiplier = difficulty / self.total_sortition_width;

        let proposer_width = multiplier * self.proposer_sortition_width;
        let transaction_width =
            multiplier * (self.proposer_sortition_width + self.tx_sortition_width);
        if hash < proposer_width {
            Some(PROPOSER_INDEX)
        } else if hash < (transaction_width + proposer_width) {
            Some(TRANSACTION_INDEX)
        } else if hash < difficulty {
            let voter_idx = (hash - proposer_width - transaction_width) % self.voter_chains.into();
            Some(voter_idx.as_u32() as u16 + FIRST_VOTER_INDEX)
        } else {
            None
        }
    }
}

// Security parameters
pub const NETWORK_DELAY: f32 = 1.4; // the expected block propagation delay (in seconds)
pub const ADVERSARY_MINING_POWER: f32 = 0.40; // the adversary power we want to tolerate
pub const LOG_EPSILON: f32 = 20.0; // -ln(1-confirmation_guarantee)
pub const ALPHA: f32 = 0.1;
// FIXME
//(VOTER_CHAIN_MINING_RATE * NETWORK_DELAY) / (1.0 + VOTER_CHAIN_MINING_RATE * NETWORK_DELAY); // alpha = orphan blocks / total blocks

lazy_static! {
    pub static ref QUANTILE_EPSILON: f32 =
        (2.0 * LOG_EPSILON - (2.0 * LOG_EPSILON).ln() - (2.0 * 3.1416926 as f32).ln()).sqrt();
    pub static ref DEFAULT_DIFFICULTY: H256 = {
        let raw: [u8; 32] = [255; 32];
        raw.into()
    };
}
