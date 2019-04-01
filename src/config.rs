// Constants

// Block sizes
pub const TX_BLOCK_SIZE: usize = 10;
pub const PROPOSER_BLOCK_SIZE: usize = 10;
pub const VOTER_BLOCK_SIZE: usize = 10;

pub const TRANSACTION_INDEX: u32 = 0;
pub const PROPOSER_INDEX: u32 = 1;
pub const FIRST_VOTER_INDEX: u32 = 2;

// Mining rates in percentages of the total mining rate
// Total for the voter chains
pub const VOTER_MINING_RATE: u32 = 10;
// Proposer tree
pub const PROPOSER_MINING_RATE: u32 = 40;
// Transaction blocks
pub const TRANSACTION_MINING_RATE: u32 =
    100 - PROPOSER_MINING_RATE - VOTER_MINING_RATE;

pub const DEFAULT_DIFFICULTY: [u8; 32] = [255; 32];
