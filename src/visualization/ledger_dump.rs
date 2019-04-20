use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::state::CoinId;
use crate::state::UTXODatabase;
use crate::transaction::Transaction;
use crate::handler;
use std::sync::Mutex;

/// Struct to display transactions/input/output human-readable
#[derive(Serialize)]
pub struct DisplayInput(String, u32);

/// Struct to display transactions/input/output human-readable
#[derive(Serialize)]
pub struct DisplayOutput(u64, String);

/// Struct to display transactions/input/output human-readable
#[derive(Serialize)]
pub struct DisplayTransaction {
    tx_hash: String,
    input: Vec<DisplayInput>,
    output: Vec<DisplayOutput>,
    //ignore signature and pub key
}

impl From<&Transaction> for DisplayTransaction {
    fn from(other: &Transaction) -> Self {
        Self {
            tx_hash: other.hash().into(),
            input: other.input.iter().map(|x|DisplayInput(x.hash.into(),x.index)).collect(),
            output: other.output.iter().map(|x|DisplayOutput(x.value,x.recipient.into())).collect(),
        }
    }
}

/// Struct to hold blockchain data to be dumped
#[derive(Serialize)]
pub struct DisplayTransactionBlock {
    /// Hash of this tx block
    pub block_hash: String,
    /// List of transactions and their hashes
    pub transactions: Vec<DisplayTransaction>, //TODO: Add tx validity
    /// List of tx hashes and list output indices which are unspent
    pub utxos: Vec<(String, Vec<usize>)>,
    // To add more fields if required
}

#[derive(Serialize)]
pub struct Dump {
    /// Ordered tx blocks
    pub transactions_blocks: Vec<DisplayTransactionBlock>,
}

pub fn dump_ledger(
    blockchain: &Mutex<BlockChain>,
    block_db: &BlockDatabase,
    state_db: &Mutex<UTXODatabase>,
) -> String {
    let blockchain = blockchain.lock().unwrap();
    let ordered_tx_block_hashes = blockchain.get_ordered_tx_blocks().clone();//why we need clone here?
    drop(blockchain);
    let mut transactions_blocks: Vec<DisplayTransactionBlock> = vec![];
    // loop over all tx blocks in the ledger
    for tx_block_hash in ordered_tx_block_hashes.iter() {
        let mut transactions = vec![];
        let mut utxos = vec![];
        let transactions_in_block: Vec<Transaction> = handler::get_tx_block_content_transactions(tx_block_hash, block_db);
        let mut utxo_state = state_db.lock().unwrap();
        // loop over all the tx in the tx_blocks
        for tx in transactions_in_block {
            let hash: H256 = tx.hash();
            //Collect the indices of unspent outputs of the tx.
            let mut unspent_indices: Vec<usize> = vec![];
            // loop over the outputs to check if they are unspent
            for (idx, _) in tx.output.iter().enumerate() {
                let coin_id = CoinId {
                    hash,
                    index: idx,
                };
                if utxo_state.check(&coin_id).unwrap() {
                    //TODO: Handle unwrap error
                    unspent_indices.push(idx);
                }
            }
            transactions.push((&tx).into());
            utxos.push((hash.into(), unspent_indices));
        }
        transactions_blocks.push(DisplayTransactionBlock {
            block_hash: tx_block_hash.into(),
            transactions,
            utxos,
        });
        drop(utxo_state);
    }
    let dump = Dump {
        transactions_blocks,
    };
    return serde_json::to_string_pretty(&dump).unwrap();
}
