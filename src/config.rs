use crate::crypto::hash::H256;
// Constants

// Block sizes
pub const TX_BLOCK_SIZE: usize = 10;
pub const PROPOSER_BLOCK_SIZE: usize = 10;
pub const VOTER_BLOCK_SIZE: usize = 10;

pub const TRANSACTION_INDEX: u32 = 1;
pub const PROPOSER_INDEX: u32 = 0;
pub const FIRST_VOTER_INDEX: u32 = 2;

// Number of chains
pub const NUM_VOTER_CHAINS: u16 = 3;

// Mining rates in percentages of the total mining rate
// Total for the voter chains
pub const VOTER_MINING_RATE: u32 = 60;
// Proposer tree
pub const PROPOSER_MINING_RATE: u32 = 20;
// Transaction blocks
pub const TRANSACTION_MINING_RATE: u32 = 100 - PROPOSER_MINING_RATE - VOTER_MINING_RATE;

pub const DEFAULT_DIFFICULTY: [u8; 32] = [255; 32];

// Genesis Hashes
lazy_static! {
    pub static ref PROPOSER_GENESIS: H256 = {
        let raw: [u8; 32] = [0; 32];
        (&raw).into()
    };
    pub static ref VOTER_GENESIS: Vec<H256> = {
        let mut v: Vec<H256> = vec![];
        for chain_num in 0..NUM_VOTER_CHAINS {
            let chain_num = chain_num as u16;
            let b1 = ((chain_num + 1) >> 8) as u8;
            let b2 = (chain_num + 1) as u8;
            let mut voter_hash_raw: [u8; 32] = [0; 32];
            voter_hash_raw[30] = b1;
            voter_hash_raw[31] = b2;
            v.push((&voter_hash_raw).into());
        }
        v
    };
}
