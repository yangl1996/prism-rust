mod proposer_block;
mod transaction;
mod voter_block;
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::config::*;
use crate::crypto::merkle::verify;
extern crate bigint;
use bigint::uint::U256;

/// The result of block validation.
pub enum BlockResult {
    /// The validation passes.
    Pass,
    /// The PoW doesn't pass.
    WrongPoW,
    /// The sortition id and content type doesn't match.
    WrongSortitionId,
    /// The content Merkle proof is incorrect.
    WrongSortitionProof,
    /// The parent block is missing.
    MissingParent(H256),
    /// Some references are missing.
    MissingReferences(Vec<H256>),
    /// Proposer Ref level > parent
    WrongProposerRef,
    /// A voter block has a out-of-range chain number.
    WrongChainNumber,
    /// A voter block votes for incorrect proposer levels.
    WrongVoteLevel,
    EmptyTransaction,
    ZeroValue,
    InsufficientInput,
    WrongSignature,
    Duplicate,
}

impl std::fmt::Display for BlockResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BlockResult::Pass => write!(f, "validation passed"),
            BlockResult::WrongPoW => write!(f, "PoW larger than difficulty"),
            BlockResult::WrongSortitionId => write!(f, "Sortition id is not same as content type"),
            BlockResult::WrongSortitionProof => write!(f, "Sortition Merkle proof is incorrect"),
            BlockResult::Duplicate => write!(f, "block already exists"),
            BlockResult::MissingParent(_) => write!(f, "parent block not in system"),
            BlockResult::MissingReferences(_) => write!(f, "referred blocks not in system"),
            BlockResult::WrongProposerRef => write!(f, "referred proposer blocks level larger than parent"),
            BlockResult::WrongChainNumber => write!(f, "chain number out of range"),
            BlockResult::WrongVoteLevel => write!(f, "incorrent vote levels"),
            BlockResult::EmptyTransaction => write!(f, "empty transaction input or output"),
            BlockResult::ZeroValue => write!(f, "transaction input or output value contains a zero"),
            BlockResult::InsufficientInput => write!(f, "insufficient input"),
            BlockResult::WrongSignature => write!(f, "signature mismatch"),
        }
    }
}

/// Validate a block.
pub fn check_block(
    block: &Block,
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
) -> BlockResult {
    // TODO: Check difficulty. Where should we get the current difficulty ranges?

    // check PoW and sortition id
    match check_pow_sortition(block, blockchain, blockdb) {
        BlockResult::Pass => {}
        x => return x,
    };

    // check whether the parent exists
    let parent = block.header.parent;
    let parent_availability = check_proposer_block_exists(parent, blockdb, blockchain);
    if !parent_availability {
        return BlockResult::MissingParent(parent);
    }

    // TODO: check timestamp

    // match the block type and check content
    match &block.content {
        Content::Proposer(content) => {
            // check whether it is a duplicate
            let self_exists = check_proposer_block_exists(block.hash(), blockdb, blockchain);
            if self_exists {
                return BlockResult::Duplicate;
            }
            // check for missing references
            let missing_refs =
                proposer_block::get_missing_references(&content, blockchain, blockdb);
            if !missing_refs.is_empty() {
                return BlockResult::MissingReferences(missing_refs);
            }
            // check refed proposer level should be less than its level
            if !proposer_block::check_ref_proposer_level(&parent, &content, blockchain) {
                return BlockResult::WrongProposerRef;
            }
            return BlockResult::Pass;
        }
        Content::Voter(content) => {
            // check whether it is a duplicate
            let self_exists = check_voter_block_exists(block.hash(), blockdb, blockchain);
            if self_exists {
                return BlockResult::Duplicate;
            }
            // check for missing references
            let missing_refs = voter_block::get_missing_references(&content, blockchain, blockdb);
            if !missing_refs.is_empty() {
                return BlockResult::MissingReferences(missing_refs);
            }
            // check chain number
            if !voter_block::check_chain_number(&content, blockchain) {
                return BlockResult::WrongChainNumber;
            }

            // check whether all proposer levels deeper than the one our parent voted are voted
            if !voter_block::check_levels_voted(&content, blockchain, &parent) {
                return BlockResult::WrongVoteLevel;
            }

            return BlockResult::Pass;
        }
        Content::Transaction(content) => {
            // check whether it is a duplicate
            let self_exists = check_transaction_block_exists(block.hash(), blockdb);
            if self_exists {
                return BlockResult::Duplicate;
            }
            // check each transaction
            for transaction in content.transactions.iter() {
                if !transaction::check_non_empty(&transaction) {
                    return BlockResult::EmptyTransaction;
                }
                if !transaction::check_non_zero(&transaction) {
                    return BlockResult::ZeroValue;
                }
// Gerui: I think we won't go to utxo in validation.
                /*
                if !transaction::check_input_unspent(&transaction, utxodb) {
                    return BlockResult::InputAlreadySpent;
                }
*/
                if !transaction::check_sufficient_input(&transaction) {
                    return BlockResult::InsufficientInput;
                }

                if !transaction::check_signature(&transaction) {
                    return BlockResult::WrongSignature;
                }
            }
            return BlockResult::Pass;
        }


    }
}

/// Check whether a proposer block exists in the block database and the blockchain.
fn check_proposer_block_exists(
    hash: H256,
    blockdb: &BlockDatabase,
    blockchain: &BlockChain,
) -> bool {
    let in_db = match blockdb.get(&hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => match b {
            None => false,
            Some(_) => true,
        },
    };

    let in_chain = match blockchain.contains_proposer(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    return in_db && in_chain;
}

/// Check whether a voter block exists in the block database and the blockchain.
fn check_voter_block_exists(hash: H256, blockdb: &BlockDatabase, blockchain: &BlockChain) -> bool {
    let in_db = match blockdb.get(&hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => match b {
            None => false,
            Some(_) => true,
        },
    };

    let in_chain = match blockchain.contains_voter(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    return in_db && in_chain;
}

/// Check whether a transaction block exists in the block database.
fn check_transaction_block_exists(hash: H256, blockdb: &BlockDatabase) -> bool {
    let in_db = match blockdb.get(&hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => match b {
            None => false,
            Some(_) => true,
        },
    };

    return in_db;
}

/// Calculate which chain should we attach the new block to
pub fn get_sortition_id(hash: &H256, difficulty: &H256) -> Option<u16> {
    let hash: [u8; 32] = hash.into();
    let big_hash = U256::from_big_endian(&hash);
    let difficulty: [u8; 32] = difficulty.into();
    let big_difficulty = U256::from_big_endian(&difficulty);
    let total_mining_range: U256 = TOTAL_MINING_RANGE.into();
    let big_proposer_range: U256 = PROPOSER_MINING_RANGE.into();
    let big_transaction_range: U256 = TRANSACTION_MINING_RANGE.into();

    if big_hash < big_difficulty / total_mining_range * big_proposer_range {
        // proposer block
        Some(PROPOSER_INDEX)
    } else if big_hash
        < big_difficulty / total_mining_range * (big_transaction_range + big_proposer_range)
    {
        // transaction block
        Some(TRANSACTION_INDEX)
    } else if big_hash < big_difficulty {
        // voter index, figure out which voter tree we are in
        let voter_id =
            (big_hash - big_transaction_range - big_proposer_range) % NUM_VOTER_CHAINS.into();
        Some(voter_id.as_u32() as u16 + FIRST_VOTER_INDEX)
    } else {
        None
    }
}

// check PoW and sortition id
pub fn check_pow_sortition(
    block: &Block,
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
) -> BlockResult {

    let sortition_id = get_sortition_id(&block.hash(), &block.header.difficulty);
    if let Some(sortition_id) = sortition_id {
        let correct_sortition_id = match &block.content {
            Content::Proposer(_) => PROPOSER_INDEX,
            Content::Transaction(_) => TRANSACTION_INDEX,
            Content::Voter(content) => content.chain_number + FIRST_VOTER_INDEX,
        };
        if sortition_id != correct_sortition_id {
            return BlockResult::WrongSortitionId;
        }
        if !verify(&block.header.content_merkle_root, &block.content.hash(), &block.sortition_proof, sortition_id as usize, (NUM_VOTER_CHAINS + FIRST_VOTER_INDEX) as usize) {
            return BlockResult::WrongSortitionProof;
        }
    } else {
        return BlockResult::WrongPoW;
    }
    BlockResult::Pass
}