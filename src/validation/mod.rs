pub mod proposer_block;
mod transaction;
mod voter_block;

use crate::block::{Block, Content, pos_metadata};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::transaction::Address;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::verify;
use crate::crypto::vrf::{vrf_verify, VrfValue};
use crate::utxodb::{UtxoDatabase, Utxo};
extern crate bigint;
use bigint::uint::U256;
use ed25519_dalek::{PublicKey, Signature};

cached! {
    U256_DIV;
    fn u256_div(dividend: U256, divisor: U256) -> U256 = {
        dividend / divisor
    }
}

/// The result of block validation.
#[derive(Debug)]
pub enum BlockResult {
    /// The validation passes.
    Pass,
    // coin not owned by public key
    WrongCoinOwner,
    //Header does not pass signature check
    WrongHeader,
    // Merkle root of the content does not match root in header
    WrongContentRoot,
    /// The coin for PoS is invalid.
    WrongCoin,
    WrongVrfProof,
    /// The PoS Vrf value doesn't pass difficulty check.
    WrongVrfValue,
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
            BlockResult::WrongCoinOwner => write!(f, "Coin does not belong to the miner"),
            BlockResult::WrongHeader => write!(f, "Header does not pass signature check"),
            BlockResult::WrongContentRoot => write!(f, "Merkle root of content does not match with header"),
            BlockResult::WrongCoin => write!(f, "Coin donen't satisfy PoS rule"),
            BlockResult::WrongVrfProof => write!(f, "PoS VRF Proof is wrong"),
            BlockResult::WrongVrfValue => write!(f, "PoS VRF value larger than difficulty"),
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

// Validate a block. Just used for testing all check_()
fn check_block(block: &Block, blockchain: &BlockChain, blockdb: &BlockDatabase) -> BlockResult {
    // TODO: Check difficulty. Where should we get the current difficulty ranges?

    /* todo fix this
    match check_pos(block) {
        // if PoW and sortition id passes, we check other rules
        BlockResult::Pass => {}
        x => return x,
    };
    */
    match check_coin_ownership(block) {
        BlockResult::Pass => {}
        x => return x,
    };
    match check_header_signature(block) {
        BlockResult::Pass => {}
        x => return x,
    };
    match check_content_hash(block) {
        BlockResult::Pass => {}
        x => return x,
    };
    match check_proof(block) {
        BlockResult::Pass => {}
        x => return x,
    };
    match check_data_availability(block, blockchain, blockdb) {
        BlockResult::Pass => {}
        x => return x,
    };
    match check_content_semantic(block, blockchain, blockdb) {
        BlockResult::Pass => {}
        x => return x,
    };
    BlockResult::Pass
}

// check the coin used to mine the block does belong to miner
pub fn check_coin_ownership(block: &Block) -> BlockResult {
    match block.header.pos_metadata.vrf_pubkey.hash() == block.header.pos_metadata.utxo.owner  {
        true => return BlockResult::Pass,
        false => return BlockResult::WrongCoinOwner,
    }   
} 

// check digital signature of block header
pub fn check_header_signature(block: &Block) -> BlockResult {
    // To check the validity of the digital signature, we need
    // 1. raw header to be signed
    // 2. public key of the coin that mines the block
    // 3. digital signature
    let mut message: Vec<&[u8]> = vec![];
    let mut signature: Vec<Signature> = vec![];
    let mut public_key: Vec<PublicKey> = vec![];

    // copy the block header with a valid signature
    let mut header = block.header.clone();
    
    // remove the signature to get the unsigned header
    header.header_signature = vec![];
    let header_unsigned = bincode::serialize(&header).unwrap();

    message.push(&header_unsigned);

    //println!("The received header signature is {:?}", block.header.header_signature);
    signature.push(Signature::from_bytes(&block.header.header_signature).unwrap());

    public_key.push((&block.header.pos_metadata.vrf_pubkey).into());

    match ed25519_dalek::verify_batch(&message, &signature, &public_key) {
        Ok(()) => return BlockResult::Pass,
        Err(_) => return BlockResult::WrongHeader,
    }        
}

//Check Merkle root of content matches with root in header
pub fn check_content_hash(block: &Block) -> BlockResult {
    match block.content.hash() == block.header.content_root  {
        true => return BlockResult::Pass,
        false => return BlockResult::WrongContentRoot,
    }   
} 

// check PoW and sortition id
pub fn check_pos(block: &Block, utxodb: &UtxoDatabase) -> BlockResult {
    // check the coin used for pos is valid
    if !utxodb.is_coin_before(&block.header.pos_metadata.utxo.coin, block.header.pos_metadata.timestamp - TAU).unwrap() {
        return BlockResult::WrongCoin;
    }
    // vrf verify
    if !vrf_verify(&block.header.pos_metadata.vrf_pubkey, &(&block.header.pos_metadata).into(), &block.header.pos_metadata.vrf_value, &block.header.pos_metadata.vrf_proof) {
        return BlockResult::WrongVrfProof;
    }
    // check vrf value
    if let Some(sortition_id) = check_difficulty(&block.header.pos_metadata.vrf_value, &block.header.difficulty, block.header.pos_metadata.utxo.value) {
        let correct_sortition_id = match &block.content {
            Content::Proposer(_) => PROPOSER_INDEX,
            Content::Transaction(_) => TRANSACTION_INDEX,
            Content::Voter(_) => PROPOSER_INDEX,
        };
        if sortition_id != correct_sortition_id {
            return BlockResult::WrongVrfValue;
        }
    } else {
        return BlockResult::WrongVrfValue;
    }
    BlockResult::Pass
}

/// check sortition proof
pub fn check_proof(block: &Block) -> BlockResult {
//        if !verify(
//            &block.header.content_merkle_root,
//            &block.content.hash(),
//            &block.proof,
//            sortition_id as usize,
//            (NUM_VOTER_CHAINS + FIRST_VOTER_INDEX) as usize,
//        ) {
//            return BlockResult::WrongSortitionProof;
//        }
    BlockResult::Pass
}
/// Validate a block that already passes pow and sortition test. See if parents/refs are missing.
pub fn check_data_availability(
    block: &Block,
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
) -> BlockResult {
    let mut missing = vec![];

    let parent = block.header.parent;

    // match the block type and check content
    match &block.content {
        Content::Proposer(content) => {
            // check whether the parent exists
            let parent_availability = check_proposer_block_exists(parent, blockchain);
            if !parent_availability {
                missing.push(parent);
            }
            // check for missing references
            let missing_refs =
                proposer_block::get_missing_references(&content, blockchain);
            if !missing_refs.is_empty() {
                missing.extend_from_slice(&missing_refs);
            }
        }
        Content::Voter(content) => {
            // check whether the parent exists
            let parent_availability = check_voter_block_exists(parent, blockchain);
            if !parent_availability {
                missing.push(parent);
            }
            // check for missing references
            let missing_refs = voter_block::get_missing_references(&content, blockchain, blockdb);
            if !missing_refs.is_empty() {
                missing.extend_from_slice(&missing_refs);
            }
        }
        Content::Transaction(_) => {
            // check whether the parent exists
            let parent_availability = check_proposer_block_exists(parent, blockchain);
            if !parent_availability {
                missing.push(parent);
            }
            // TODO: note that we don't care about blockdb here, since all blocks at this stage
            // should have been inserted into the blockdb
        }
    }

    if !missing.is_empty() {
        return BlockResult::MissingReferences(missing);
    } else {
        return BlockResult::Pass;
    }
}

/// Check block content semantic
pub fn check_content_semantic(
    block: &Block,
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
) -> BlockResult {
    let parent = block.header.parent;
    match &block.content {
        Content::Proposer(content) => {
            // check refed proposer level should be less than its level
            if !proposer_block::check_ref_proposer_level(&parent, &content, blockchain) {
                return BlockResult::WrongProposerRef;
            }
            return BlockResult::Pass;
        }
        Content::Voter(content) => {
            // check chain number
            if !voter_block::check_chain_number(&parent, &content, blockchain) {
                return BlockResult::WrongChainNumber;
            }
            // check whether all proposer levels deeper than the one our parent voted are voted
            if !voter_block::check_levels_voted(&content, blockchain, &parent) {
                return BlockResult::WrongVoteLevel;
            }
            return BlockResult::Pass;
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

                if !transaction::check_num_authorizations(&transaction) {
                    return BlockResult::WrongSignature; // this is not quite precise
                }
            }
            if !transaction::check_signature_batch(&content.transactions) {
                return BlockResult::WrongSignature;
            }
            return BlockResult::Pass;
        }
    }
}

/// Check whether a proposer block exists in the block database and the blockchain.
fn check_proposer_block_exists(hash: H256, blockchain: &BlockChain) -> bool {
    let in_chain = match blockchain.contains_proposer(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    return in_chain;
}

/// Check whether a voter block exists in the block database and the blockchain.
fn check_voter_block_exists(hash: H256, blockchain: &BlockChain) -> bool {
    let in_chain = match blockchain.contains_voter(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    return in_chain;
}

/// Check whether a transaction block exists in the block database.
fn check_transaction_block_exists(hash: H256, blockchain: &BlockChain) -> bool {
    let in_chain = match blockchain.contains_transaction(&hash) {
        Err(e) => panic!("Blockchain error {}", e),
        Ok(b) => b,
    };

    return in_chain;
}

/// Calculate which chain should we attach the new block to
/// Returns Some(chain_id) where chain_id=0 for proposer or voter, chain_id=1 for transaction block
/// Returns None for not passing difficulty validation
pub fn check_difficulty(hash: &VrfValue, difficulty: &H256, stake: u64) -> Option<u16> {
    let hash: [u8; 32] = hash.into();
    let big_hash = U256::from_big_endian(&hash);
    let difficulty: [u8; 32] = difficulty.into();
    let big_difficulty = U256::from_big_endian(&difficulty);
    let big_difficulty = big_difficulty * stake.into();//TODO: relative stake
    let big_proposer_range: U256 = PROPOSER_MINING_RANGE.into();
    let big_transaction_range: U256 = TRANSACTION_MINING_RANGE.into();
    let total_mining_range: U256 = big_proposer_range + big_transaction_range;
    // *DEFAULT_DIFFICULTY_DIV
    // big_difficulty / total_mining_range
    if big_hash < u256_div(big_difficulty, total_mining_range) * big_proposer_range {
        // proposer block
        Some(PROPOSER_INDEX)
    } else if big_hash < big_difficulty {
        // transaction block
        Some(TRANSACTION_INDEX)
    } else {
        // Didn't pass PoS
        None
    }
}



#[cfg(test)]
mod tests {
    use super::super::config::*;
    use super::check_difficulty;
    use crate::crypto::hash::H256;

    #[test]
    fn sortition_id() {
        let difficulty = *DEFAULT_DIFFICULTY;
        let hash: H256 = [0; 32].into();
        assert_eq!(check_difficulty(&hash, &difficulty, 1), Some(PROPOSER_INDEX));
        // This hash should fail PoW test (so result is None)
        assert_eq!(check_difficulty(&difficulty, &difficulty, 1), None);
    }
}
