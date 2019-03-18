/*
It randomly generates objects of the given class
*/

use super::{Block, Content};
use super::header::Header;
use super::transaction::Content as tx_Content;
use super::proposer::Content as proposer_Content;
use super::voter::Content as voter_Content;

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{Transaction,  Input, Output, Signature};

use crate::crypto::generator as crypto_generator;
use crate::transaction::generator as tx_generator;

use rand::{Rng, RngCore};


pub fn header() -> Header {
    let mut rng = rand::thread_rng();
    let parent_hash = crypto_generator::H256();
    let timestamp = rng.next_u64();
    let nonce = rng.next_u32();
    let content_root = crypto_generator::H256();
    let extra_content = crypto_generator::u8_32_array();
    let difficulty = crypto_generator::u8_32_array();
    return Header::new(parent_hash, timestamp, nonce, content_root, extra_content, difficulty)
}

pub fn tx_content()  -> tx_Content {
    let mut rng = rand::thread_rng();
    let tx_number =  rng.gen_range(1, 10);
    let transactions :Vec<Transaction> = (0..tx_number).map(|_| tx_generator::transaction()).collect();
    return tx_Content {transactions};
}
pub fn tx_block() -> Block{
    let header = header();
    let content = Content::Transaction(tx_content());
    let sortition_proof :Vec<H256> = (0..10).map(|_| crypto_generator::H256()).collect();
    return Block{header, content, sortition_proof};
}

pub fn tx_block_with_parent_hash(parent_hash: H256) -> Block{
    let mut tx_block = tx_block();
    tx_block.header.parent_hash = parent_hash;
    return tx_block;
}


pub fn tx_blocks_with_parent_hash(num: u32, parent_hash: H256) -> Vec<Block> {
    return (0..num).map( |_| tx_block_with_parent_hash(parent_hash)).collect();
}
/// no references to prop blocks.
pub fn proposer_content1(transaction_block_hashes: Vec<H256>) -> proposer_Content{
    let proposer_block_hashes :Vec<H256> = vec![];
    return proposer_Content {transaction_block_hashes, proposer_block_hashes};
}
pub fn prop_block1(transaction_block_hashes: Vec<H256>) -> Block{
    let header = header();
    let proposer_content = proposer_content1(transaction_block_hashes);
    let content = Content::Proposer(proposer_content);
    let sortition_proof :Vec<H256> = (0..10).map(|_| crypto_generator::H256()).collect();
    return Block{header, content, sortition_proof};
}

pub fn prop_block1_with_parent_hash(parent_hash: H256, transaction_block_hashes: Vec<H256>) -> Block{
    let mut prop_block = tx_block();
    prop_block.header.parent_hash = parent_hash;
    return prop_block;
}

/// has references to prop blocks.
pub fn proposer_content2() -> proposer_Content{
    let transaction_block_hashes :Vec<H256> = (0..5).map(|_| tx_block().hash()).collect();
    let proposer_block_hashes :Vec<H256> = vec![];
    return proposer_Content {transaction_block_hashes, proposer_block_hashes};
}
pub fn prop_block2() -> Block{
    let header = header();
    let content = Content::Proposer(proposer_content2());
    let sortition_proof :Vec<H256> = (0..10).map(|_| crypto_generator::H256()).collect();
    return Block{header, content, sortition_proof};
}

//pub fn voter_content(chain_number: u16, proposer_block_votes: Vec<H256>) -> voter_Content{
//    let voter_parent_hash = crypto_generator::H256();
//    return voter_Content {chain_number, voter_parent_hash, proposer_block_votes};
//}
//
//pub fn voter_block(chain_number: u16) -> Block{
//    let header = header();
//    let content = Content::Voter(voter_content(chain_number));
//    let sortition_proof :Vec<H256> = (0..10).map(|_| crypto_generator::H256()).collect();
//    return Block{header, content, sortition_proof};
//}

//pub fn mining(tx_content: tx_Content, proposer_content: proposer_Content,
//            mut voter_content: voter_Content, index: u16) ->  Block { //for now all voter contents are the same
//    let mut content_hash_vec : Vec<H256> = vec![];
//
//    let m=10;
//    /// Adding voter content
//    for i in 0..m {
//        voter_content.chain_number = i as u16;
//        content_hash_vec.push(voter_content.hash());
//    }
//
//    /// Adding transaction content
//    let transaction_content = tx_content;
//    content_hash_vec.push(transaction_content.hash());
//
//    /// Adding proposer content
//    let proposer_content = proposer_content;
//    content_hash_vec.push(proposer_content.hash());
//
//    let merkle_tree = MerkleTree::new(&content_hash_vec);
//    let mut header = header();
//    header.content_root = *merkle_tree.root();
//
//    /// Fake mining: The content corresponding to 'index' is chosen
//    let content: Content;
//    if index < m {
//        let mut voter_content = voter_content;
//        voter_content.chain_number = index as u16;
//        content = Content::Voter(voter_content.clone());
//    }
//    else if index ==  m {
//        content = Content::Transaction(transaction_content);
//    }
//    else{
//        content = Content::Proposer(proposer_content);
//    }
//
//    let sortition_proof = merkle_tree.get_proof_from_index(index as  u32);
//    let sortition_proof = sortition_proof.iter().map(|&x| *x).collect();
//
//    return Block::from_header(header, content, sortition_proof);
//}
