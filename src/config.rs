use crate::crypto::hash::H256;
use crate::block::pos_metadata::{RandomSource, TimeStamp};
use bigint::uint::U256;

// Time step for pos, in millisecond
pub const DELTA: TimeStamp = 100;
// Time tau for pos, only coin older than t-tau will be used
pub const TAU: TimeStamp = 20000;
// The extra buffer time for holding spent coins
pub const TAU_NETWORK_DELAY: TimeStamp = 10000;
// Time u for pos, only proposer after t-u can be voted
pub const OMEGA: TimeStamp = 400000;
// Time s for pos, compare voter chain length using s-truncated rule
pub const SIGMA: TimeStamp = 400000;
// Level c for pos. change random source using c-correlated rule
pub const GAMMA: u64 = 5;

// Network parameters
pub const NETWORK_DELAY: f32 = 2.0; // the expected block propagation delay (in seconds)

// Design parameters
pub const NUM_VOTER_CHAINS: u16 = 100 as u16; // more chains means better latency
pub const TX_BLOCK_SIZE: u32 = 64000; // the maximum size of a transaction block (in Bytes)
pub const TX_THROUGHPUT: u32 = 70; // the transaction throughput we want to support (in Tx/s)
pub const TX_BLOCK_TRANSACTIONS: u32 = TX_BLOCK_SIZE / AVG_TX_SIZE;
pub const PROPOSER_BLOCK_TX_REFS: u32 = (TX_MINING_RATE / CHAIN_MINING_RATE * 2.0) as u32;

pub const AVG_TX_SIZE: u32 = 280; // average size of a transaction (in Bytes)
pub const TX_MINING_RATE: f32 = TX_THROUGHPUT as f32 / TX_BLOCK_TRANSACTIONS as f32;
pub const CHAIN_MINING_RATE: f32 = 0.2 / NETWORK_DELAY; // mining rate of the proposer chain and each voter chain in Blks/s

// Do not change from here

// Mining rate of each type (Proposer : Voter (all chains) : Transaction, in Blks/s)
pub const RATIO: (f32, f32, f32) = (
    CHAIN_MINING_RATE,
    CHAIN_MINING_RATE * (NUM_VOTER_CHAINS as f32),
    TX_MINING_RATE,
);

// Sortition ranges
pub const TOTAL_MINING_RANGE: u32 = 10000000; // This is for resolution
pub const RATE_DIFFICULTY_MULTIPLIER: f32 =
    (TOTAL_MINING_RANGE as f32) / (RATIO.0 + RATIO.1 + RATIO.2);

// Width of the acceptance range for each type of block
pub const VOTER_MINING_RANGE: u32 = (RATE_DIFFICULTY_MULTIPLIER * RATIO.1) as u32;
pub const PROPOSER_MINING_RANGE: u32 = (RATE_DIFFICULTY_MULTIPLIER * RATIO.0) as u32;
pub const TRANSACTION_MINING_RANGE: u32 = (RATE_DIFFICULTY_MULTIPLIER * RATIO.2) as u32;

// Chain id
pub const TRANSACTION_INDEX: u16 = 1;
pub const PROPOSER_INDEX: u16 = 0;
pub const FIRST_VOTER_INDEX: u16 = 1;

lazy_static! {
    pub static ref DEFAULT_DIFFICULTY: H256 = {
        let mut raw: [u8; 32] = [255; 32];
        raw[0]=0x00;
        raw[1]=0x00;
        raw[2]=0xdf;
        raw.into()
    };

    pub static ref DEFAULT_DIFFICULTY_DIV: U256 = {
        let difficulty: [u8; 32] = (&*DEFAULT_DIFFICULTY).into();
        let big_difficulty = U256::from_big_endian(&difficulty);
        let big_proposer_range: U256 = PROPOSER_MINING_RANGE.into();
        let big_transaction_range: U256 = TRANSACTION_MINING_RANGE.into();
        let total_mining_range: U256 = big_proposer_range + big_transaction_range;
        big_difficulty / total_mining_range 
    };

    // Genesis Hashes
    pub static ref PROPOSER_GENESIS_HASH: H256 = {
        let raw: [u8; 32] = [0; 32];
        raw.into()
    };
    pub static ref VOTER_GENESIS_HASHES: Vec<H256> = {
        let mut v: Vec<H256> = vec![];
        for chain_num in 0..NUM_VOTER_CHAINS {
            let chain_num = chain_num as u16;
            let b1 = ((chain_num + 1) >> 8) as u8;
            let b2 = (chain_num + 1) as u8;
            let mut voter_hash_raw: [u8; 32] = [0; 32];
            voter_hash_raw[30] = b1;
            voter_hash_raw[31] = b2;
            v.push(voter_hash_raw.into());
        }
        v
    };
    // Genesis RandomSource
    pub static ref PROPOSER_GENESIS_RAND: RandomSource= {
        let raw: [u8; 32] = [0; 32];
        raw
    };
    pub static ref VOTER_GENESIS_RANDS: Vec<RandomSource> = {
        let mut v: Vec<RandomSource> = vec![];
        for chain_num in 0..NUM_VOTER_CHAINS {
            let chain_num = chain_num as u16;
            let b1 = ((chain_num + 1) >> 8) as u8;
            let b2 = (chain_num + 1) as u8;
            let mut voter_hash_raw: [u8; 32] = [0; 32];
            voter_hash_raw[30] = b1;
            voter_hash_raw[31] = b2;
            v.push(voter_hash_raw);
        }
        v
    };
    // Max number of votes cast by a voter block.
    pub static ref VOTER_BLOCK_VOTES_LIMIT: u32 = {
        let log_no_voter_chains = (NUM_VOTER_CHAINS as f64).ln();
        3*(log_no_voter_chains.ceil() as u32)
    };

}
