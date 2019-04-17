use crate::block::transaction::Content as TxContent;
use crate::block::{Block, Content};
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::state::UTXODatabase;
use crate::state::{CoinData, CoinId, UTXO};
use crate::transaction::{Transaction};

use std::sync::Mutex;

/// This function changes the ledger to incorporate txs from last 'tx_block_hashes'.
pub fn confirm_new_tx_blocks(
    tx_block_hashes: Vec<H256>,
    block_db: &BlockDatabase,
    state_db: &Mutex<UTXODatabase>,//TODO: add wallets, when state_db receive/rollback, wallet also receive/rollback.
) {
    //1. Loop over the tx blocks
    for tx_block_hash in tx_block_hashes.iter() {
        let tx_block: Block = block_db
            .get(tx_block_hash)
            .unwrap_or(panic!("TX block not found in DB. 1"))
            .unwrap_or(panic!("TX block not found in DB. 2"));
        let transactions = match tx_block.content {
            Content::Transaction(content) => content.transactions,
            _ => panic!("Wrong block stored"),
        };
        let tmp:Vec<(Vec<CoinId>, Vec<UTXO>)> = transactions.iter().map(|tx|to_utxo(tx)).collect();
        //2. Loop over the transactions
        let utxo_state = state_db.lock().unwrap();
        {
            for transaction in transactions.iter() {
                //3a. Sanitize: Check if all the inputs are unspent (in the state)
                let mut inputs_unspent = true;
                for input in transaction.input.iter() {
                    match utxo_state.check(&(input.into())) {
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
                    match utxo_state.receive(transaction) {
                        Err(e) => panic!("StateDB not working: Error {}", e),
                        Ok(_) => (),
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
pub fn unconfirm_old_tx_blocks(
    tx_block_hashes: Vec<H256>, // These blocks must be the tip of the ordered tx blocks.
    block_db: &BlockDatabase,
    state_db: &Mutex<UTXODatabase>,
) {
    //1. Loop over the tx blocks in reverse
    for tx_block_hash in tx_block_hashes.iter().rev() {
        let tx_block: Block = block_db
            .get(tx_block_hash)
            .unwrap_or(panic!("TX block not found in DB. 1"))
            .unwrap_or(panic!("TX block not found in DB. 2"));
        let transactions = match tx_block.content {
            Content::Transaction(content) => content.transactions,
            _ => panic!("Wrong block stored"),
        };
        //2. Loop over the transactions
        let utxo_state = state_db.lock().unwrap();
        {
            for transaction in transactions.iter().rev() {
                // also need rev?
                let hash: H256 = transaction.hash();
                //3a. Revert the transaction only if it was valid when it was added in the state.
                // Logic: If the transaction was valid, *all* its output should be unspent/present in the state.
                let output_size = transaction.output.len();
                let mut no_unspent_outputs: usize = 0;
                for index in 0..output_size {
                    let coin_id = CoinId {
                        hash,
                        index: index as u32,
                    };
                    match utxo_state.check(&coin_id) {
                        Err(e) => panic!("StateDB not working: Error {}", e),
                        Ok(unspent) => {
                            if unspent {
                                no_unspent_outputs += 1;
                            }
                        }
                    }
                }

                //3b. State transition: If the transaction was valid, then delete all the outputs and add back all the inputs
                if no_unspent_outputs == output_size {
                    match utxo_state.rollback(transaction) {
                        Err(e) => panic!("StateDB not working: Error {}", e),
                        Ok(_) => (),
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

// future: use this function in receive
pub fn to_utxo(tx: &Transaction) -> (Vec<CoinId>, Vec<UTXO>) {
    let hash: H256 = tx.hash(); // compute hash here, and below inside Input we don't have to compute again (we just copy)
    // i) delete all the input coins
    let to_delete: Vec<CoinId> = tx.input.iter().map(|input|input.into()).collect();
    // ii) add all output coins to the state
    let to_add: Vec<UTXO> = tx.output.iter().enumerate().map(|(index, output)|
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
    (to_delete, to_add)
}

// future: use this function in rollback
pub fn to_rollback_utxo(tx: &Transaction) -> (Vec<CoinId>, Vec<UTXO>) {
    let hash: H256 = tx.hash();
    // i) Get the input locations of the output coins and delete the output coins.
    let to_delete: Vec<CoinId> = (0..(tx.output.len() as u32)).map(|index|
        CoinId {
            hash,
            index: index as u32,
        }).collect();
    // ii) Reconstruct the input utxos
    let to_add: Vec<UTXO> = tx.input.iter().map(|input|
        UTXO {
            coin_id: input.into(),
            coin_data: CoinData {
                value: input.value,
                recipient: input.recipient,
            },
        }).collect();
    (to_delete, to_add)
}
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::state::generator as state_generator;
//    use crate::transaction::generator as tx_generator;
//    use crate::crypto::generator as crypto_generator;
//    use crate::transaction::Transaction;
//    use crate::state::UTXODatabase;
//
//
//    #[test]
//    pub fn confirm_unconfirm(){
//        //1. init database
//        let default_path = "/tmp/state_db_rocksdb";
//        let statedb_path = std::path::Path::new(&default_path);
//        let statedb = UTXODatabase::new(statedb_path).unwrap();
//        let tx_block = crypto_generator::h256();
//
//        //2. Add a few transactions  to begin with
//        let mut transactions: Vec<Transaction> = vec![];
//        for position in 0..20 {
//            let transaction = tx_generator::random();
//            for utxo in state_generator::tx_to_utxos(&transaction) {
//                statedb.insert(&utxo);
//            }
//            transactions.push(transaction);
//        }
//        println!("{}", transactions[0]);
//
//    }
//}

//TODO: Add tests
