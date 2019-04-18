use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::state::UTXODatabase;
use crate::crypto::hash::H256;
use crate::transaction::{Transaction};
use crate::state::{CoinId};

/// Struct to hold blockchain data to be dumped
#[derive(Serialize)]

struct DisplayTransactionBlock {
    /// List of transactions and list output indices which are unspent
    pub transactions: Vec<(Transaction, Vec<usize>)>, //TODO: Add tx validity
    // To add more fields if required
}

pub struct Dump {
    /// Ordered tx blocks
    pub transactions_blocks: Vec<DisplayTransactionBlock>
}


pub fn dump_ledger(blockchain: &BlockChain, blockdb: &BlockDatabase, statedb: &UTXODatabase) -> String {
    let ordered_tx_block_hashes = blockchain.get_ordered_tx_blocks();
    let mut transactions_blocks: Vec<DisplayTransactionBlock>;
    // loop over all tx blocks in the ledger
    for tx_hash in ordered_tx_block_hashes.iter() {
        let tx_block = blockdb.get(tx_hash).unwrap().unwrap(); //TODO: Handle unwrap errors
        let mut transactions: Vec<(Transaction, Vec<usize>)>;
        // loop over all the tx in the tx_blocks
        for tx in tx_block {
            let hash: H256 = tx.hash();
            //Collect the indices of unspent outputs of the tx.
            let unspent_indices: Vec<u64>;
            // loop over the outputs to check if they are unspent
            for (idx, _output) in tx.output().iter().enumerate(){
                let coin_id = CoinId {
                    hash,
                    index: idx as u32,
                };
                // @Gerui: use data from _output to get recipient and value if required.
                if statedb.get(coin_id).unwrap() { //TODO: Handle unwrap error
                    unspent_indices.push(idx);
                }
             transactions.push((tx, unspent_indices));
            }
        transactions_blocks.push(DisplayTransactionBlock{transactions});
        }
    }
    let dump = Dump {transactions_blocks};
    return serde_json::to_string_pretty(&dump).unwrap();
}
