use super::header::Header;
use super::proposer::Content as proposer_Content;
use super::transaction::Content as tx_Content;
use super::voter::Content as voter_Content;
use super::{Block, Content};

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::transaction::{Input, Output, Signature, Transaction};

macro_rules! gen_hashed_data {
    () => {{
        vec![
            (&hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
            (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into(),
            (&hex!("0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b")).into(),
            (&hex!("0403020108070605040302010807060504030201080706050403020108070605")).into(),
            (&hex!("1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a")).into(),
            (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into(),
            (&hex!("0000000100000001000000010000000100000001000000010000000100000001")).into(),
        ]
    }};
}

// Header stuff
pub fn sample_header() -> Header {
    let parent_hash: H256 =
        (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into();
    let timestamp: u64 = 7094730;
    let nonce: u32 = 839782;
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

pub fn sample_header_hash_should_be() -> H256 {
    let header_hash_should_be =
        (&hex!("db7136134bbb9df6fbc46a43e0d27a42d11d460f74d0f57ce6ddbfaf96e386db")).into(); // Calculated on Mar 15, 2019
    return header_hash_should_be;
}

// Transaction block stuff
/// Returns sample content of a transaction block
pub fn sample_transaction_content() -> tx_Content {
    let hashed_data: Vec<H256> = gen_hashed_data!();

    let input1 = Input {
        hash: hashed_data[0],
        index: 1,
    };
    let input2 = Input {
        hash: hashed_data[1],
        index: 2,
    };
    let input_vec: Vec<Input> = vec![input1, input2];

    let output1 = Output {
        recipient: hashed_data[2],
        value: 10,
    };
    let output2 = Output {
        recipient: hashed_data[3],
        value: 12,
    };
    let output_vec: Vec<Output> = vec![output1, output2];

    let signature_vec: Vec<Signature> = vec![];

    let sample_transaction = Transaction {
        input: input_vec,
        output: output_vec,
        signatures: signature_vec,
    };

    let mut transaction_vec: Vec<Transaction> = vec![];
    transaction_vec.push(sample_transaction.clone());
    transaction_vec.push(sample_transaction.clone());
    transaction_vec.push(sample_transaction.clone());

    let transaction_content = tx_Content {
        transactions: transaction_vec,
    };
    return transaction_content;
}
pub fn sample_transaction_content_hash_shouldbe() -> H256 {
    let transaction_content_hash: H256 =
        (&hex!("9591ba700494f707a4dde613c278bda050fbbfa45d217a5b1e5f5d30f4cec2d3")).into();
    return transaction_content_hash;
}

/// Returns sample a transaction block
pub fn sample_transaction_block() -> Block {
    let transaction_content: tx_Content = sample_transaction_content();
    let header = sample_header(); // The content root is incorrect
    let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
    return Block {
        header,
        content: Content::Transaction(transaction_content),
        sortition_proof,
    };
}

// Proposer block stuffs
/// Returns sample content of a proposer block containing only tx block hashes
pub fn sample_proposer_content1() -> proposer_Content {
    let tx_block = sample_transaction_block();
    let transaction_block_hashes = vec![tx_block.hash()];

    let proposer_block_content = proposer_Content {
        transaction_block_hashes,
        proposer_block_hashes: vec![],
    };
    let header = sample_header();
    let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
    return proposer_block_content;
}
pub fn sample_proposer_content1_hash_shouldbe() -> H256 {
    let transaction_content_hash: H256 =
        (&hex!("92b44c4a1f245d603d9bf8befdd09cd0921f0cbfdfc00772c2e0073bd6145669")).into();
    return transaction_content_hash;
}

/// Returns sample content of a proposer block containing only tx block hashes and prop block hashes
pub fn sample_proposer_content2() -> proposer_Content {
    let tx_block = sample_transaction_block();
    let transaction_block_hashes = vec![tx_block.hash()];
    let propose_block = sample_proposer_block1();
    let proposer_block_hashes = vec![propose_block.hash()];
    let proposer_block_content = proposer_Content {
        transaction_block_hashes,
        proposer_block_hashes,
    };
    return proposer_block_content;
}
pub fn sample_proposer_content2_hash_shouldbe() -> H256 {
    let transaction_content_hash: H256 =
        (&hex!("92b44c4a1f245d603d9bf8befdd09cd0921f0cbfdfc00772c2e0073bd6145669")).into();
    return transaction_content_hash;
}

/// Returns sample a proposer block 1 and 2
pub fn sample_proposer_block1() -> Block {
    let proposer_block_content = sample_proposer_content1();
    let header = sample_header(); // The content root is incorrect
    let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
    return Block {
        header,
        content: Content::Proposer(proposer_block_content),
        sortition_proof,
    };
}
pub fn sample_proposer_block2() -> Block {
    let proposer_block_content = sample_proposer_content2();
    let mut header = sample_header();
    header.content_root = sample_proposer_content1_hash_shouldbe();
    let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
    return Block {
        header,
        content: Content::Proposer(proposer_block_content),
        sortition_proof,
    };
}

// Voter block stuff
pub fn sample_voter_content() -> voter_Content {
    let chain_number = 0;
    let voter_parent_hash =
        (&hex!("0000000100000001000000010000000100000001000000010000000100000001")).into();
    let proposer_block1 = sample_proposer_block1();
    let proposer_block2 = sample_proposer_block2();
    let proposer_block_votes = vec![proposer_block1.hash(), proposer_block2.hash()];
    return voter_Content::new(chain_number, voter_parent_hash, proposer_block_votes);
}
pub fn sample_voter_content1_hash_shouldbe() -> H256 {
    let transaction_content_hash: H256 =
        (&hex!("102a65b93fbf3a56bc73299d0ecb81d44c442bdf975baf57ded034b809d19fd1")).into();
    return transaction_content_hash;
}

// The block with valid sortition proof is mined.
pub fn sample_mined_block(index: u32) -> Block {
    let m: u32 = 100; // number of  voter trees
    let mut content_hash_vec: Vec<H256> = vec![];

    // Adding voter content
    for i in 0..m {
        let mut voter_content = sample_voter_content();
        voter_content.chain_number = i as u16;
        content_hash_vec.push(voter_content.hash());
    }

    // Adding transaction content
    let transaction_content = sample_transaction_content();
    content_hash_vec.push(transaction_content.hash());

    // Adding proposer content
    let proposer_content = sample_proposer_content2();
    content_hash_vec.push(proposer_content.hash());

    let merkle_tree = MerkleTree::new(&content_hash_vec);
    let mut header = sample_header();
    header.content_root = merkle_tree.root();

    // Fake mining: The content corresponding to 'index' is chosen
    let content: Content;
    if index < m {
        let mut voter_content = sample_voter_content();
        voter_content.chain_number = index as u16;
        content = Content::Voter(voter_content.clone());
    } else if index == m {
        content = Content::Transaction(transaction_content);
    } else {
        content = Content::Proposer(proposer_content);
    }

    let sortition_proof = merkle_tree.get_proof_from_index(index);

    return Block::from_header(header, content, sortition_proof);
}
