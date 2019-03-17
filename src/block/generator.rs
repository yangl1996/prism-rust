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
use crate::crypto::generator as crypto_generator;
use crate::transaction::generator as tx_generator;
use crate::transaction::{Transaction,  Input, Output, Signature};
use rand::{Rng, RngCore};
type rgen = rand::prelude::ThreadRng;


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
    let tx_size =  rng.gen_range(1, 10);
    let transactions :Vec<Transaction> = (0..tx_size).map(|_| tx_generator::transaction()).collect();
    return tx_Content {transactions};
}