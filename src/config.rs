use crate::crypto::hash::H256;

// Network parameters
pub const NETWORK_CAPACITY: u32 = 500_000; // 0.5 MB/s == 4Mb/s. We will use 80% of capacity for the txs.
pub const NETWORK_DELAY: f32 = 2.0; // 2 sec delay
// Design parameters
pub const NUM_VOTER_CHAINS: u16 = 10 as u16; //
pub const TX_BLOCK_SIZE_BYTES: u32 = 64_000; //64KB

// All the parameters below are function of the above parameters
pub const TX_THROUGHPUT: u32 = NETWORK_CAPACITY*4/5; // .4 MB == 3.2 Mb.
pub const CHAIN_MINING_RATE: f32 = 0.2/(NETWORK_DELAY); // Mining rate of each chain.

//Ratio Prop::Total_Voter::Tx_block
pub const RATIO: (f32, f32, f32) = (1.0, NUM_VOTER_CHAINS as f32, (TX_THROUGHPUT as f32)/((TX_BLOCK_SIZE_BYTES as f32)*CHAIN_MINING_RATE) ); 

// Mining rates in percentages*100 of the total mining rate
pub const TOTAL_MINING_RANGE: u32 = 10000; // This is only used for resolution
pub const CHAIN_MINING_RANGE: u32 = ((TOTAL_MINING_RANGE as f32)/(RATIO.0+RATIO.1+RATIO.2)) as u32;
// Total for the voter chains
pub const VOTER_MINING_RANGE: u32 = CHAIN_MINING_RANGE * (NUM_VOTER_CHAINS as u32);
// Proposer tree
pub const PROPOSER_MINING_RANGE: u32 = CHAIN_MINING_RANGE;
// Transaction blocks
pub const TRANSACTION_MINING_RANGE: u32 = TOTAL_MINING_RANGE - PROPOSER_MINING_RANGE - VOTER_MINING_RANGE;

//Block content size limits
pub const AVG_TX_SIZE_BYTES: u32 = 500;
pub const TRANSACTION_BLOCK_TX_LIMIT: u32 = TX_BLOCK_SIZE_BYTES/AVG_TX_SIZE_BYTES; // Max number of tx included in a tx block
pub const PROPOSER_BLOCK_TX_BLOCK_REF_LIMIT: u32 = 3*TRANSACTION_MINING_RANGE/PROPOSER_MINING_RANGE; // Max number of tx blocks referred by a prop block.
pub const PROPOSER_BLOCK_PROP_BLOCK_REF_LIMIT: u32 = 10; // Max number of prop blocks referred by a prop block.

// Chain id
pub const TRANSACTION_INDEX: u32 = 1;
pub const PROPOSER_INDEX: u32 = 0;
pub const FIRST_VOTER_INDEX: u32 = 2;

lazy_static! {
    pub static ref DEFAULT_DIFFICULTY: H256 = {
        let mut raw: [u8; 32] = [255; 32];
        raw.into()
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
    // Max number of votes cast by a voter block.
    pub static ref VOTER_BLOCK_VOTES_LIMIT: u32 = {
        let log_no_voter_chains = (NUM_VOTER_CHAINS as f64).ln();
        3*(log_no_voter_chains.ceil() as u32)
    };

}
