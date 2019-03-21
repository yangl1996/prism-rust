/*
It randomly generates objects of the given class
*/

use super::{Block, Content};
use super::header::Header;
use super::transaction::Content as Tx_Content;
use super::proposer::Content as Proposer_Content;
use super::voter::Content as Voter_Content;

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{Transaction,  Input, Output, Signature};

use crate::crypto::generator as crypto_generator;
use crate::transaction::generator as tx_generator;

use rand::{Rng, RngCore};

/// Returns a random header (with randomly filled fields)
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

/// Returns a random tx_content filled with 1-10 transactions
fn tx_content()  -> Tx_Content {
    let mut rng = rand::thread_rng();
    let tx_number =  rng.gen_range(1, 10);
    let transactions :Vec<Transaction> = (0..tx_number).map(|_| tx_generator::transaction()).collect();
    return Tx_Content {transactions};
}

/// Returns a random tx_block with 10 length sortition proof.
pub fn tx_block() -> Block{
    let header = header();
    let content = Content::Transaction(tx_content());
    let sortition_proof :Vec<H256> = (0..10).map(|_| crypto_generator::H256()).collect();
    return Block{header, content, sortition_proof};
}


//pub fn voter_content(chain_number: u16, proposer_block_votes: Vec<H256>) -> Voter_Content{
//    let voter_parent_hash = crypto_generator::H256();
//    return Voter_Content {chain_number, voter_parent_hash, proposer_block_votes};
//}
//
//pub fn voter_block(chain_number: u16) -> Block{
//    let header = header();
//    let content = Content::Voter(voter_content(chain_number));
//    let sortition_proof :Vec<H256> = (0..10).map(|_| crypto_generator::H256()).collect();
//    return Block{header, content, sortition_proof};
//}

//pub fn mining(tx_content: tx_Content, proposer_content: Proposer_Content,
//            mut voter_content: Voter_Content, index: u16) ->  Block { //for now all voter contents are the same
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
