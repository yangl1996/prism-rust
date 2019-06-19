use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use crate::miner::memory_pool::MemoryPool;
use crate::network::message;
use crate::network::server::Handle as ServerHandle;
use crate::transaction::Transaction;
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use std::sync::Mutex;

pub fn update_ledger(
    blockdb: &BlockDatabase,
    chain: &BlockChain,
    utxodb: &UtxoDatabase,
    wallet: &Wallet,
) {
    let diff = chain.update_ledger().unwrap();
    PERFORMANCE_COUNTER.record_confirm_transaction_blocks(diff.0.len());
    PERFORMANCE_COUNTER.record_deconfirm_transaction_blocks(diff.1.len());

    // gather the transaction diff and apply on utxo database
    let mut add: Vec<Transaction> = vec![];
    let mut remove: Vec<Transaction> = vec![];
    for hash in diff.0 {
        let block = blockdb.get(&hash).unwrap().unwrap();
        let content = match block.content {
            Content::Transaction(data) => data,
            _ => unreachable!(),
        };
        let mut transactions = content.transactions.clone();
        add.append(&mut transactions);
    }
    for hash in diff.1 {
        let block = blockdb.get(&hash).unwrap().unwrap();
        let content = match block.content {
            Content::Transaction(data) => data,
            _ => unreachable!(),
        };
        let mut transactions = content.transactions.clone();
        remove.append(&mut transactions);
    }

    let coin_diff = utxodb.apply_diff(&add, &remove, diff.2).unwrap();
    wallet.apply_diff(&coin_diff.0, &coin_diff.1).unwrap();
}
