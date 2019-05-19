use crate::crypto::hash::H256;

// Block sizes
pub const TX_BLOCK_SIZE: usize = 10;
pub const PROPOSER_BLOCK_SIZE: usize = 10;
pub const VOTER_BLOCK_SIZE: usize = 10;

// chain id
pub const TRANSACTION_INDEX: u32 = 1;
pub const PROPOSER_INDEX: u32 = 0;
pub const FIRST_VOTER_INDEX: u32 = 2;


// 1:100:100 - Ratio
// Number of voter chains
pub const NUM_VOTER_CHAINS: u16 = 100;

// Mining rates in percentages of the total mining rate
pub const CHAIN_MINING_RATE: u32 = 50;
// Total for the voter chains
pub const VOTER_MINING_RATE: u32 = CHAIN_MINING_RATE*(NUM_VOTER_CHAINS as u32);
// Proposer tree
pub const PROPOSER_MINING_RATE: u32 = CHAIN_MINING_RATE;
// Transaction blocks
pub const TRANSACTION_MINING_RATE: u32 = 10000 - PROPOSER_MINING_RATE - VOTER_MINING_RATE;

//Block content limits
pub const TRANSACTION_BLOCK_TX_LIMIT: u32 = 400; //Max number of tx included in a tx  block
pub const PROPOSER_BLOCK_TX_BLOCK_REF_LIMIT: u32 = 300; // Max number of tx blocks referred by a prop block.
pub const PROPOSER_BLOCK_PROP_BLOCK_REF_LIMIT: u32 = 10; // Max number of prop blocks referred by a prop block.

lazy_static! {
    pub static ref DEFAULT_DIFFICULTY: H256 = {
        let mut raw: [u8; 32] = [255; 32];
        raw[0] = 0;
        raw[1] = 25;
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
}
