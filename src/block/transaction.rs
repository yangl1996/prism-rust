use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::transaction::Transaction;
use crate::experiment::performance_counter::PayloadSize;

/// The content of a transaction block.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    pub transactions: Vec<Transaction>, // TODO: No coinbase transaction for now
}

impl Content {
    /// Create new transaction block content.
    pub fn new(transactions: Vec<Transaction>) -> Self {
        Self { transactions }
    }
}

impl PayloadSize for Content {
    fn size(&self) -> usize {
        let mut total = 0;
        for t in &self.transactions {
            total += t.size();
        }
        return total;
    }
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        // TODO: we are hashing txs in a merkle tree.
        let merkle_tree = MerkleTree::new(&self.transactions);
        return merkle_tree.root();
    }
}



#[cfg(test)]
pub mod tests {
    use super::super::header::tests::*;
    use super::super::transaction::Content as TxContent;
    use crate::crypto::sign::{KeyPair, PubKey, Signable, Signature};
    use crate::crypto::hash::{Hashable, H256};
    use crate::crypto::merkle::MerkleTree;
    use crate::transaction::{CoinId, Input, Output, Authorization, Transaction};
    use std::cell::RefCell;
    use super::super::{Block, Content};



    ///The hash should match
    #[test]
    fn test_hash() {
        let block = sample_transaction_block();
        let block_hash_should_be = sample_transaction_content_hash_shouldbe();
        assert_eq!(block.hash(), block_hash_should_be);
    }


    macro_rules! gen_hashed_data {
        () => { {
            vec ! [
            ( & hex ! ("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
            ( & hex ! ("0102010201020102010201020102010201020102010201020102010201020102")).into(),
            ( & hex ! ("0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b")).into(),
            ( & hex ! ("0403020108070605040302010807060504030201080706050403020108070605")).into(),
            ]
        }};
    }
    macro_rules! gen_owner_data {
        () => {{
            vec![
                (&hex!("1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a")).into(),
                (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into(),
                (&hex!("0000000100000001000000010000000100000001000000010000000100000001")).into(),
            ]
        }};
    }

    // Transaction block stuff
    /// Returns sample content of a transaction block
    pub fn sample_transaction_content() -> TxContent {
        let hashed_data: Vec<H256> = gen_hashed_data!();
        let owner_data: Vec<H256> = gen_owner_data!();

        let coin_id_1 = CoinId{
            hash: hashed_data[0],
            index: 1,
        };
        let input1 = Input {
            coin: coin_id_1,
            value: 9,
            owner: owner_data[0], //or other hash?
        };

        let coin_id_2 = CoinId{
            hash: hashed_data[1],
            index: 1,
        };
        let input2 = Input {
            coin: coin_id_2,
            value: 13,
            owner: owner_data[1], //or other hash?
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

        let random_vec: Vec<u8> = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 130, 160,188, 245, 62, 73, 135, 240, 180, 20, 177, 255, 98, 37, 238, 80, 71, 211, 5, 13, 4, 227, 175, 11, 134, 142, 194, 93, 73, 97, 87, 243, 161, 35, 3, 33, 0, 25, 224, 252, 17, 218, 195, 88, 253, 142, 89, 193, 92, 198, 154, 38, 231, 160, 220, 87, 38, 107, 94, 100, 183, 161, 185, 36, 254, 158, 188, 45, 170].to_vec();
        let keypair = KeyPair::from_pkcs8(random_vec);
        let unsigned_transaction = Transaction {
            input: input_vec,
            output: output_vec,
            authorization: vec![],
            hash: RefCell::new(None)
        };

        let authorization = vec![
            Authorization {
                pubkey: keypair.public_key(),
                signature: unsigned_transaction.sign(&keypair),
            }];

        let transaction_vec: Vec<Transaction> = vec![Transaction {
            authorization,
            ..unsigned_transaction
        }];

        let transaction_content = TxContent {
            transactions: transaction_vec,
        };
        return transaction_content;
    }
    pub fn sample_transaction_content_hash_shouldbe() -> H256 {
        let transaction_content_hash: H256 =
            (&hex!("1d6e0f5f11248d070c6b15d103ead72a8802279c600a60b40b38510b582aacbf")).into();
        return transaction_content_hash;
    }
    /// Returns sample a transaction block
    pub fn sample_transaction_block() -> Block {
        let transaction_content: TxContent = sample_transaction_content();
        let header = sample_header(); // The content root is incorrect
        let sortition_proof: Vec<H256> = vec![]; // The sortition proof is bogus
        return Block {
            header,
            content: Content::Transaction(transaction_content),
            sortition_proof,
        };
    }

}
