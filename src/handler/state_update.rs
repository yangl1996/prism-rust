use crate::block::transaction::Content as TxContent;
use crate::block::{Block, Content};
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::state::{UTXODatabase, CoinData, CoinId, UTXO};
use crate::transaction::{Transaction, generator as tx_generator};
use crate::wallet::Wallet;
use std::sync::{Mutex, Arc};
use rand::{Rng, RngCore};


/// This function changes the ledger to incorporate txs from last 'tx_block_hashes'.
pub fn confirm_new_tx_block_hashes (
    tx_block_hashes: Vec<H256>,
    block_db: &BlockDatabase,
    state_db: &Mutex<UTXODatabase>,//do we need a mutex here?
    wallets: &Vec<Arc<Mutex<Wallet>>>,
) {
    let tx_block_transactions: Vec<Vec<Transaction>> = tx_block_hashes.iter().map(|hash|get_tx_block_content_transactions(hash, block_db)).collect();
    confirm_new_tx_block_transactions(tx_block_transactions, state_db, wallets);
}

pub fn confirm_new_tx_block_transactions(
    tx_block_transactions: Vec<Vec<Transaction>>,
    state_db: &Mutex<UTXODatabase>,//do we need a mutex here?
    wallets: &Vec<Arc<Mutex<Wallet>>>,
) {
    //1. Loop over the tx block's transactionss
    for transactions in tx_block_transactions {
        // pre-compute the utxos to be deleted and inserted
        let to_delete_insert: Vec<(Vec<CoinId>, Vec<UTXO>)> = transactions.iter().map(|tx|to_utxo(tx)).collect();
        //2. Loop over the transactions
        let mut utxo_state = state_db.lock().unwrap();
        {
            for (to_delete, to_insert) in to_delete_insert.iter() {
                //3a. Sanitize: Check if all the inputs are unspent (in the state)
                let mut inputs_unspent = true;
                for coin_id in to_delete {
                    match utxo_state.check(coin_id) {
                        Err(e) => panic!("State DB not working: Error {}", e),
                        Ok(unspent) => {
                            if !unspent {
                                inputs_unspent = false;
                                break;
                            }
                        }
                    }
                }
                //3b. State transition: If all inputs are unspent, then receive this transaction
                if inputs_unspent {
                    match utxo_state.update(to_delete, to_insert) {
                        Err(e) => panic!("StateDB not working: Error {}", e),
                        Ok(_) => (),
                    }
                    for wallet in wallets {
                        let mut w = wallet.lock().unwrap();
                        w.update(to_delete, to_insert);
                    }
                } else {
                    //log the sanitization error.
                }
            }
        }
        drop(utxo_state); //@Lei: is this required?
    }
}

/// This function removes the ledger changes from last 'tx_block_hashes'.
pub fn unconfirm_old_tx_block_hashes (
    tx_block_hashes: Vec<H256>, // These blocks must be the tip of the ordered tx blocks.
    block_db: &BlockDatabase,
    state_db: &Mutex<UTXODatabase>,
    wallets: &Vec<Arc<Mutex<Wallet>>>,
) {
    // note: we have a rev() here
    let tx_block_transactions: Vec<Vec<Transaction>> = tx_block_hashes.iter().rev().map(|hash|get_tx_block_content_transactions(hash, block_db)).collect();
    unconfirm_old_tx_block_transactions(tx_block_transactions, state_db, wallets);
}
pub fn unconfirm_old_tx_block_transactions(
    tx_block_transactions: Vec<Vec<Transaction>>, // These blocks must be the tip of the ordered tx blocks.
    state_db: &Mutex<UTXODatabase>,
    wallets: &Vec<Arc<Mutex<Wallet>>>,
) {
    //1. Loop over the tx block's transactions (already in reverse order)
    for transactions in tx_block_transactions {
        // pre-compute the utxos to be deleted and inserted
        let to_delete_insert: Vec<(Vec<CoinId>, Vec<UTXO>)> = transactions.iter().map(|tx|to_rollback_utxo(tx)).collect();
        //2. Loop over the transactions
        let mut utxo_state = state_db.lock().unwrap();
        {
            for (to_delete, to_insert) in to_delete_insert.iter().rev() {//need rev here?
                //3a. Revert the transaction only if it was valid when it was added in the state.
                // Logic: If the transaction was valid, *all* its output should be unspent/present in the state.
                let mut no_unspent_outputs: usize = 0;
                for coin_id in to_delete {
                    match utxo_state.check(coin_id) {
                        Err(e) => panic!("StateDB not working: Error {}", e),
                        Ok(unspent) => {
                            if unspent {
                                no_unspent_outputs += 1;
                            }
                        }
                    }
                }
                //3b. State transition: If the transaction was valid, then delete all the outputs and add back all the inputs
                if no_unspent_outputs == to_delete.len() {
                    match utxo_state.update(to_delete, to_insert) {
                        Err(e) => panic!("StateDB not working: Error {}", e),
                        Ok(_) => (),
                    }
                    for wallet in wallets {
                        let mut w = wallet.lock().unwrap();
                        w.update(to_delete, to_insert);
                    }
                } else if no_unspent_outputs == 0 {
                    //log the sanitization error.
                } else {
                    panic!("Partial sanitization should not occur!");
                }
            }
        }
        drop(utxo_state);
    }
}

fn get_tx_block_content_transactions(hash: &H256, block_db: &BlockDatabase) -> Vec<Transaction> {
    let tx_block: Block = block_db
        .get(hash)
        .unwrap_or(panic!("TX block not found in DB. 1"))
        .unwrap_or(panic!("TX block not found in DB. 2"));
    let transactions = match tx_block.content {
        Content::Transaction(content) => content.transactions,
        _ => panic!("Wrong block stored"),
    };
    transactions
}

/// convert a transaction to two vectors of utxos, first is to be deleted from state, second is to be inserted
pub fn to_utxo(tx: &Transaction) -> (Vec<CoinId>, Vec<UTXO>) {
    let hash: H256 = tx.hash(); // compute hash here, and below inside Input we don't have to compute again (we just copy)
    // i) delete all the input coins
    let to_delete: Vec<CoinId> = tx.input.iter().map(|input|input.into()).collect();
    // ii) add all output coins to the state
    let to_insert: Vec<UTXO> = tx.output.iter().enumerate().map(|(index, output)|
        UTXO {
            coin_id: CoinId {
                hash,
                index: index as u32,
            },
            coin_data: CoinData {
                value: output.value,
                recipient: output.recipient,
            },
        }).collect();
    (to_delete, to_insert)
}

/// Reverse version of to_utxo. When rollback, convert a transaction to two vectors of utxos, first is to be deleted from state, second is to be inserted
pub fn to_rollback_utxo(tx: &Transaction) -> (Vec<CoinId>, Vec<UTXO>) {
    let hash: H256 = tx.hash();
    // i) Get the input locations of the output coins and delete the output coins.
    let to_delete: Vec<CoinId> = (0..(tx.output.len() as u32)).map(|index|
        CoinId {
            hash,
            index: index as u32,
        }).collect();
    // ii) Reconstruct the input utxos
    let to_insert: Vec<UTXO> = tx.input.iter().map(|input|
        UTXO {
            coin_id: input.into(),
            coin_data: CoinData {
                value: input.value,
                recipient: input.recipient,
            },
        }).collect();
    (to_delete, to_insert)
}

// Tests are in tests/state_update.rs
