use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::state::CoinId;
use crate::state::UTXODatabase;
use crate::transaction::Transaction;

use std::sync::Mutex;

/// Struct to hold blockchain data to be dumped
#[derive(Serialize)]
pub struct DisplayTransactionBlock {
    /// List of transactions and list output indices which are unspent
    pub transactions: Vec<(Transaction, Vec<usize>)>, //TODO: Add tx validity
                                                      // To add more fields if required
}

#[derive(Serialize)]
pub struct Dump {
    /// Ordered tx blocks
    pub transactions_blocks: Vec<DisplayTransactionBlock>,
}

pub fn dump_ledger(
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
    state_db: &Mutex<UTXODatabase>,
) -> String {
    let ordered_tx_block_hashes = blockchain.get_ordered_tx_blocks();
    let mut transactions_blocks: Vec<DisplayTransactionBlock> = vec![];
    // loop over all tx blocks in the ledger
    for tx_hash in ordered_tx_block_hashes.iter() {
        let tx_block = blockdb.get(tx_hash).unwrap().unwrap(); //TODO: Handle unwrap errors
        let mut display_transactions: Vec<(Transaction, Vec<usize>)>;
        let transactions = match tx_block.content {
            Content::Transaction(content) => content.transactions,
            _ => panic!("Wrong block stored"),
        };
        let mut utxo_state = state_db.lock().unwrap();
        // loop over all the tx in the tx_blocks
        for tx in transactions {
            let hash: H256 = tx.hash();
            //Collect the indices of unspent outputs of the tx.
            let mut unspent_indices: Vec<usize>;
            // loop over the outputs to check if they are unspent
            for (idx, _output) in tx.output.iter().enumerate() {
                let coin_id = CoinId {
                    hash,
                    index: idx as u32,
                };
                // @Gerui: use data from _output to get recipient and value if required.
                if utxo_state.check(&coin_id).unwrap() {
                    //TODO: Handle unwrap error
                    unspent_indices.push(idx);
                }
            }
            display_transactions.push((tx, unspent_indices));
        }
        transactions_blocks.push(DisplayTransactionBlock {
            transactions: display_transactions,
        });
        drop(utxo_state);
    }
    let dump = Dump {
        transactions_blocks,
    };
    return serde_json::to_string_pretty(&dump).unwrap();
}
