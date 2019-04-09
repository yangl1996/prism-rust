use crate::block::transaction::Content as TxContent;
use crate::block::{Block, Content};
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::state::UTXODatabase;
use crate::state::{CoinId, UTXO};
use crate::transaction::Transaction;

use std::sync::Mutex;

/// Sanitize the transaction blocks and modify the state
pub fn confirm_new_tx_blocks(
    tx_block_hashes: Vec<H256>,
    block_db: &BlockDatabase,
    utxo_state: &Mutex<UTXODatabase>,
) -> Vec<H256> {
    let mut sanitized_tx_block_hashes: Vec<H256> = vec![];

    //1. Loop over the tx blocks
    for tx_block_hash in tx_block_hashes.iter() {
        let mut sanitized_transactions: Vec<Transaction> = vec![];

        let tx_block: Block = block_db
            .get(tx_block_hash)
            .unwrap_or(panic!("TX block not found in DB 1"))
            .unwrap_or(panic!("TX block not found in DB 2"));
        let transactions = match tx_block.content {
            Content::Transaction(content) => content.transactions,
            _ => panic!("Wrong block stored"),
        };
        //2. Loop over the transactions
        let state = utxo_state.lock().unwrap();
        {
            for transaction in transactions.iter() {
                //3a. Sanitize: Check if all the inputs are present in the state
                let mut inputs_unspent = true;
                for input in transaction.input.iter() {
                    match state.check(input) {
                        Err(e) => panic!("StateDB not working: Error {}", e),
                        Ok(present) => {
                            if !present {
                                inputs_unspent = false;
                                break;
                            }
                        }
                    }
                }

                //3b. State transition: If all inputs are unspent, then delete the input coins and add output coins to the state
                if inputs_unspent {
                    for input in transaction.input.iter() {
                        state.delete(input);
                    }
                    for (index, output) in transaction.output.iter().enumerate() {
                        let coin_id = CoinId {
                            hash: transaction.hash(),
                            index: index as u32,
                        };
                        let utxo = UTXO {
                            coin_id,
                            value: output.value,
                        };
                        state.insert(&utxo);
                    }
                    sanitized_transactions.push(*transaction);
                } else {
                    //log the sanitization error.
                }
            }
        }
        drop(state);

        // Construct the sanitized blocks
        let tx_content = Content::Transaction(TxContent {
            transactions: sanitized_transactions,
        });
        let mut header = tx_block.header;
        header.nonce += 1; //TODO: Bad code: Changing the header slightly to modify it hash value.
        let sanitized_block = Block {
            header: header,
            content: tx_content,
            sortition_proof: vec![],
        };
        sanitized_tx_block_hashes.push(sanitized_block.hash());
        block_db.insert(&sanitized_block);
    }
    return sanitized_tx_block_hashes;
}

pub fn unconfirm_old_tx_blocks(
    sanitized_tx_blocks: Vec<H256>,
    db: &BlockDatabase,
    state: &Mutex<UTXODatabase>,
) {
    unimplemented!();
}

//TODO: Add tests
