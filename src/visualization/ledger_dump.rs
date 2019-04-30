use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::handler;
use crate::state::UTXODatabase;
use crate::transaction::CoinId;
use crate::transaction::Transaction as RawTransaction;

#[derive(Serialize)]
pub struct Input {
    hash: String,
    index: u32,
}

#[derive(Serialize)]
pub struct Output {
    value: u64,
    recipient: String,
}

#[derive(Serialize)]
pub struct Transaction {
    hash: String,
    input: Vec<Input>,
    output: Vec<Output>,
}

#[derive(Serialize)]
pub struct TransactionBlock {
    /// Hash of this tx block
    pub hash: String,
    /// List of transactions
    pub transactions: Vec<Transaction>, //TODO: Add tx validity
    /// List of tx hashes and list output indices which are unspent
    pub utxos: Vec<Input>,
}

#[derive(Serialize)]
pub struct Dump {
    /// Ordered tx blocks
    pub transactions_blocks: Vec<TransactionBlock>,
}

pub fn dump_ledger(
    blockchain: &BlockChain,
    block_db: &BlockDatabase,
    state_db: &UTXODatabase,
) -> String {
    let ordered_tx_block_hashes = blockchain.get_ordered_tx_blocks();
    let mut transactions_blocks: Vec<TransactionBlock> = vec![];

    // loop over all tx blocks in the ledger
    for tx_block_hash in ordered_tx_block_hashes.iter() {
        let mut transactions = vec![];
        let mut utxos = vec![];
        let transactions_in_block: Vec<RawTransaction> =
            handler::get_tx_block_content_transactions(tx_block_hash, block_db);

        // loop over all the tx in this transaction block
        for tx in transactions_in_block {
            let hash: H256 = tx.hash();
            // loop over the outputs to check if they are unspent
            for index in 0..tx.output.len() {
                let coin_id = CoinId {
                    hash,
                    index: index as u32,
                };
                if let Ok(unspent) = state_db.check(&coin_id) {
                    if unspent {
                        utxos.push(Input {
                            hash: hash.to_string(),
                            index: index as u32,
                        });
                    }
                }
            }

            // add this transaction to the list
            transactions.push(Transaction {
                hash: hash.to_string(),
                input: tx
                    .input
                    .iter()
                    .map(|x| Input {
                        hash: x.coin.hash.to_string(),
                        index: x.coin.index,
                    })
                    .collect(),
                output: tx
                    .output
                    .iter()
                    .map(|x| Output {
                        value: x.value,
                        recipient: x.recipient.to_string(),
                    })
                    .collect(),
            });
        }
        transactions_blocks.push(TransactionBlock {
            hash: tx_block_hash.to_string(),
            transactions,
            utxos,
        });
    }
    let dump = Dump {
        transactions_blocks,
    };
    return serde_json::to_string_pretty(&dump).unwrap();
}
