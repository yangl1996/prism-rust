mod proposer_block;
mod transaction;
mod voter_block;
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::utxodb::UtxoDatabase;

/// The result of block validation.
pub enum BlockResult {
    /// The validation passes.
    Pass,
    /// The parent block is missing.
    MissingParent(H256),
    /// The content hash does not match.
    WrongContentHash,
    /// Some references are missing.
    MissingReferences(Vec<H256>),
    /// A voter block has a out-of-range chain number.
    WrongChainNumber,
    /// A voter block votes for incorrect proposer levels.
    WrongVoteLevel,
    EmptyTransaction,
    InputAlreadySpent,
    InsufficientInput,
    WrongSignature,
    Duplicate,
}

impl std::fmt::Display for BlockResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BlockResult::Pass => write!(f, "validation passed"),
            BlockResult::Duplicate => write!(f, "block already exists"),
            BlockResult::MissingParent(_) => write!(f, "parent block not in system"),
            BlockResult::WrongContentHash => write!(f, "content hash mismatch"),
            BlockResult::MissingReferences(_) => write!(f, "referred blocks not in system"),
            BlockResult::WrongChainNumber => write!(f, "chain number out of range"),
            BlockResult::WrongVoteLevel => write!(f, "incorrent vote levels"),
            BlockResult::EmptyTransaction => write!(f, "empty transaction input or output"),
            BlockResult::InputAlreadySpent => write!(f, "input already spent"),
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
    utxodb: &UtxoDatabase,
) -> BlockResult {
    // TODO: check PoW. Where should we get the current difficulty ranges?

    // check whether the parent exists
    let parent = block.header.parent;
    let parent_availability = check_proposer_block_exists(parent, blockdb, blockchain);
    if !parent_availability {
        return BlockResult::MissingParent(parent);
    }

    // TODO: check timestamp
    // TODO: check sortition proof

    /*
    // check the content hash
    if block.content.hash() != block.header.content_root {
        return BlockResult::WrongContentHash;
    }
    */

    // match the block type and check content
    match &block.content {
        Content::Transaction(content) => {
            // check each transaction
            /*
            for transaction in content.transactions.iter() {
                /*
                if !transaction::check_non_empty(&transaction) {
                    return BlockResult::EmptyTransaction;
                }

                if !transaction::check_input_unspent(&transaction, utxodb) {
                    return BlockResult::InputAlreadySpent;
                }

                if !transaction::check_sufficient_input(&transaction) {
                    return BlockResult::InsufficientInput;
                }

                if !transaction::check_signature(&transaction) {
                    return BlockResult::WrongSignature;
                }
                */
            }
            */
            return BlockResult::Pass;
        }
        Content::Proposer(content) => {
            // check for missing references
            let missing_refs =
                proposer_block::get_missing_references(&content, blockchain, blockdb);
            if missing_refs.len() == 0 {
                return BlockResult::Pass;
            } else {
                return BlockResult::MissingReferences(missing_refs);
            }
        }
        Content::Voter(content) => {
            // check for missing references
            let missing_refs = voter_block::get_missing_references(&content, blockchain, blockdb);
            if missing_refs.len() != 0 {
                return BlockResult::MissingReferences(missing_refs);
            }
            // TODO: if those two checks are not disabled, stack overflow.
            /*
            // check chain number
            if !voter_block::check_chain_number(&content) {
                return BlockResult::WrongChainNumber;
            }

            // check whether all proposer levels deeper than the one our parent voted are voted
            if !voter_block::check_levels_voted(&content, blockchain, blockdb, parent) {
                return BlockResult::WrongVoteLevel;
            }
            */
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
    let in_db = match blockdb.contains(&hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => b,
    };

    let in_chain = match blockchain.contains_proposer(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    return in_db && in_chain;
}

/// Check whether a voter block exists in the block database and the blockchain.
fn check_voter_block_exists(hash: H256, blockdb: &BlockDatabase, blockchain: &BlockChain) -> bool {
    let in_db = match blockdb.contains(&hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => b
    };

    let in_chain = match blockchain.contains_voter(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    return in_db && in_chain;
}

/// Check whether a transaction block exists in the block database.
fn check_transaction_block_exists(hash: H256, blockdb: &BlockDatabase) -> bool {
    let in_db = match blockdb.contains(&hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => b
    };

    return in_db;
}
