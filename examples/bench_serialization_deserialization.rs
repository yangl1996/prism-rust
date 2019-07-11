use prism::block::header::Header;
use prism::block::transaction::Content as TxContent;
use prism::block::{Block, Content};
use prism::crypto::hash::{Hashable, H256};
use prism::transaction::Transaction;
#[macro_use]
extern crate hex_literal;
use bincode::{deserialize, serialize};
use log::{debug, error, info};
use std::process;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const REPEAT: usize = 200;
const TX_BLOCK_TX_COUNT: usize = 500;

#[cfg(feature = "test-utilities")]
fn main() {
    use prism::transaction::tests::generate_random_transaction;
    let start = Instant::now();
    let mut blocks: Vec<Block> = vec![];
    let start = Instant::now();

    for nonce in 0..REPEAT {
        let mut txs: Vec<Transaction> = vec![];
        let header = sample_header(nonce as u32); // The content root is incorrect
        for i in 0..TX_BLOCK_TX_COUNT {
            txs.push(generate_random_transaction());
        }
        let transaction_content = TxContent { transactions: txs };
        let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
        let block = Block {
            header,
            content: Content::Transaction(transaction_content),
            sortition_proof,
        };
        blocks.push(block);
    }

    let end = Instant::now();
    let time = end.duration_since(start).as_micros() as f64;
    println!(
        "Block generation  rate {}",
        (REPEAT as f64) * 1000000.0 / time
    );

    let mut serialized_blocks: Vec<Vec<u8>> = vec![];
    let start = Instant::now();
    for i in 0..REPEAT {
        let b = serialize(&blocks[i]).unwrap();
        serialized_blocks.push(b);
    }
    let end = Instant::now();
    let time = end.duration_since(start).as_micros() as f64;
    println!(
        "Block serialization rate rate {}",
        (REPEAT as f64) * 1000000.0 / time
    );

    let mut deserialized_blocks: Vec<Block> = vec![];
    let start = Instant::now();
    for i in 0..REPEAT {
        let b: Block = deserialize(&serialized_blocks[i]).unwrap();
        deserialized_blocks.push(b);
    }
    let end = Instant::now();
    let time = end.duration_since(start).as_micros() as f64;
    println!(
        "Block deserialization rate rate {}",
        (REPEAT as f64) * 1000000.0 / time
    );
}

// Header stuff
pub fn sample_header(nonce: u32) -> Header {
    let parent_hash: H256 =
        (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into();
    let timestamp: u128 = 7094730;
    let content_root: H256 =
        (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into();
    let extra_content: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let difficulty: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        20, 10,
    ];
    let difficulty = (&difficulty).into();
    let header = Header::new(
        parent_hash,
        timestamp,
        nonce,
        content_root,
        extra_content,
        difficulty,
    );
    return header;
}
