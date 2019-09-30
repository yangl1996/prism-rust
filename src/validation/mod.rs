mod proposer_block;
mod transaction;
mod voter_block;
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::verify;
extern crate bigint;

/// The result of block validation.
#[derive(Debug)]
pub enum BlockResult {
    /// The validation passes.
    Pass,
    /// The PoW doesn't pass.
    WrongPoW,
    /// The sortition id and content type doesn't match.
    WrongSortitionId,
    /// The content Merkle proof is incorrect.
    WrongSortitionProof,
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
}

impl std::fmt::Display for BlockResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BlockResult::Pass => write!(f, "validation passed"),
            BlockResult::WrongPoW => write!(f, "PoW larger than difficulty"),
            BlockResult::WrongSortitionId => write!(f, "Sortition id is not same as content type"),
            BlockResult::WrongSortitionProof => write!(f, "Sortition Merkle proof is incorrect"),
            BlockResult::MissingReferences(_) => write!(f, "referred blocks not in system"),
            BlockResult::WrongProposerRef => {
                write!(f, "referred proposer blocks level larger than parent")
            }
            BlockResult::WrongChainNumber => write!(f, "chain number out of range"),
            BlockResult::WrongVoteLevel => write!(f, "incorrent vote levels"),
            BlockResult::EmptyTransaction => write!(f, "empty transaction input or output"),
            BlockResult::ZeroValue => {
                write!(f, "transaction input or output value contains a zero")
            }
            BlockResult::InsufficientInput => write!(f, "insufficient input"),
            BlockResult::WrongSignature => write!(f, "signature mismatch"),
        }
    }
}

// check PoW and sortition id
pub fn check_pow_sortition_id(block: &Block, config: &BlockchainConfig) -> BlockResult {
    let sortition_id = config.sortition_hash(&block.hash(), &block.header.difficulty);
    if let Some(sortition_id) = sortition_id {
        let correct_sortition_id = match &block.content {
            Content::Proposer(_) => PROPOSER_INDEX,
            Content::Transaction(_) => TRANSACTION_INDEX,
            Content::Voter(content) => content.chain_number + FIRST_VOTER_INDEX,
        };
        if sortition_id != correct_sortition_id {
            return BlockResult::WrongSortitionId;
        }
    } else {
        return BlockResult::WrongPoW;
    }
    BlockResult::Pass
}

/// check sortition proof
pub fn check_sortition_proof(block: &Block, config: &BlockchainConfig) -> BlockResult {
    let sortition_id = config.sortition_hash(&block.hash(), &block.header.difficulty);
    if let Some(sortition_id) = sortition_id {
        if !verify(
            &block.header.content_merkle_root,
            &block.content.hash(),
            &block.sortition_proof,
            sortition_id as usize,
            (config.voter_chains + FIRST_VOTER_INDEX) as usize,
        ) {
            return BlockResult::WrongSortitionProof;
        }
    } else {
        unreachable!();
    }
    BlockResult::Pass
}
/// Validate a block that already passes pow and sortition test. See if parents/refs are missing.
pub fn check_data_availability(
    block: &Block,
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
) -> BlockResult {
    let mut missing = vec![];

    // check whether the parent exists
    let parent = block.header.parent;
    let parent_availability = check_proposer_block_exists(parent, blockchain);
    if !parent_availability {
        missing.push(parent);
    }

    // match the block type and check content
    match &block.content {
        Content::Proposer(content) => {
            // check for missing references
            let missing_refs =
                proposer_block::get_missing_references(&content, blockchain, blockdb);
            if !missing_refs.is_empty() {
                missing.extend_from_slice(&missing_refs);
            }
        }
        Content::Voter(content) => {
            // check for missing references
            let missing_refs = voter_block::get_missing_references(&content, blockchain, blockdb);
            if !missing_refs.is_empty() {
                missing.extend_from_slice(&missing_refs);
            }
        }
        Content::Transaction(_) => {
            // TODO: note that we don't care about blockdb here, since all blocks at this stage
            // should have been inserted into the blockdb
        }
    }

    if !missing.is_empty() {
        BlockResult::MissingReferences(missing)
    } else {
        BlockResult::Pass
    }
}

/// Check block content semantic
pub fn check_content_semantic(
    block: &Block,
    blockchain: &BlockChain,
    _blockdb: &BlockDatabase,
) -> BlockResult {
    let parent = block.header.parent;
    match &block.content {
        Content::Proposer(content) => {
            // check refed proposer level should be less than its level
            if !proposer_block::check_ref_proposer_level(&parent, &content, blockchain) {
                return BlockResult::WrongProposerRef;
            }
            BlockResult::Pass
        }
        Content::Voter(content) => {
            // check chain number
            if !voter_block::check_chain_number(&content, blockchain) {
                return BlockResult::WrongChainNumber;
            }
            // check whether all proposer levels deeper than the one our parent voted are voted
            if !voter_block::check_levels_voted(&content, blockchain, &parent) {
                return BlockResult::WrongVoteLevel;
            }
            BlockResult::Pass
        }
        Content::Transaction(content) => {
            // check each transaction
            for transaction in content.transactions.iter() {
                if !transaction::check_non_empty(&transaction) {
                    return BlockResult::EmptyTransaction;
                }
                if !transaction::check_non_zero(&transaction) {
                    return BlockResult::ZeroValue;
                }
                if !transaction::check_sufficient_input(&transaction) {
                    return BlockResult::InsufficientInput;
                }
            }
            if !transaction::check_signature_batch(&content.transactions) {
                return BlockResult::WrongSignature;
            }
            BlockResult::Pass
        }
    }
}

/// Check whether a proposer block exists in the block database and the blockchain.
fn check_proposer_block_exists(hash: H256, blockchain: &BlockChain) -> bool {
    let in_chain = match blockchain.contains_proposer(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    in_chain
}

/// Check whether a voter block exists in the block database and the blockchain.
fn check_voter_block_exists(hash: H256, blockchain: &BlockChain) -> bool {
    let in_chain = match blockchain.contains_voter(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    in_chain
}

/// Check whether a transaction block exists in the block database.
fn check_transaction_block_exists(hash: H256, blockchain: &BlockChain) -> bool {
    let in_chain = match blockchain.contains_transaction(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    in_chain
}
