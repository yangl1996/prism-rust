mod transaction;
mod proposer_block;
mod voter_block;
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::state::UTXODatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::Transaction;
use std::sync::Mutex;

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
}

/// Validate a block.
pub fn check_block(block: &Block, blockchain: &Mutex<BlockChain>, blockdb: &BlockDatabase, utxodb: &UTXODatabase) -> BlockResult {
    // TODO: check PoW. Where should we get the current difficulty ranges?

    // check whether the parent exists
    let parent = block.header.parent_hash;
    let parent_availability = check_block_exist(parent, blockchain, blockdb);
    if !(parent_availability.0 && parent_availability.1) {
        return BlockResult::MissingParent(parent);
    }

    // TODO: check timestamp
    // TODO: check sortition proof

    // check the content hash
    if block.content.hash() != block.header.content_root {
        return BlockResult::WrongContentHash;
    }

    // match the block type and check content
    match &block.content {
        Content::Transaction(content) => {
            // check each transaction
            for transaction in content.transactions.iter() {
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
            }
            return BlockResult::Pass;
        }
        Content::Proposer(content) => {
            // check for missing references
            let missing_refs = proposer_block::get_missing_references(&content, blockchain, blockdb);
            if missing_refs.len() == 0 {
                return BlockResult::Pass;
            }
            else {
                return BlockResult::MissingReferences(missing_refs);
            }
        }
        Content::Voter(content) => {
            // check for missing references
            let missing_refs = voter_block::get_missing_references(&content, blockchain, blockdb);
            if missing_refs.len() != 0 {
                return BlockResult::MissingReferences(missing_refs);
            }

            // check chain number
            if !voter_block::check_chain_number(&content) {
                return BlockResult::WrongChainNumber;
            }

            // check whether all proposer levels deeper than the one our parent voted are voted
            if !voter_block::check_levels_voted(&content, blockchain, blockdb) {
                return BlockResult::WrongVoteLevel;
            }

            return BlockResult::Pass;
        }
    }
}

/// Check whether a block exists in the blockchain and the block database. The function returns a
/// tuple, of which the first member being whether the block is in the block database, and the
/// second one the block chain.
fn check_block_exist(hash: H256, blockchain: &Mutex<BlockChain>, blockdb: &BlockDatabase) -> (bool, bool) {
    let in_db = match blockdb.get(&hash) {
        Err(e) => panic!("Database error {}", e),
        Ok(b) => match b {
            None => false,
            Some(_) => true,
        }
    };

    let in_blockchain = blockchain.lock().unwrap().check_node(hash);

    return (in_db, in_blockchain);
}
