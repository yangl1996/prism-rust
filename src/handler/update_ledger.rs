use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::miner::memory_pool::MemoryPool;
use crate::network::message;
use crate::network::server::Handle as ServerHandle;
use crate::transaction::{Transaction, Input};
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use std::sync::Mutex;
use crate::visualization::demo::Server as DemoServer;

pub fn update_transaction_sequence (
    blockdb: &BlockDatabase,
    chain: &BlockChain,
    demo_server: &DemoServer,
) -> (Vec<(Transaction, H256)>, Vec<(Transaction, H256)>) {
    let diff = chain.update_ledger().unwrap();

    //send confirmed blocks to demo
    demo_server.update_ledger(&diff.2, &diff.3).unwrap();

    PERFORMANCE_COUNTER.record_confirm_transaction_blocks(diff.0.len());
    PERFORMANCE_COUNTER.record_deconfirm_transaction_blocks(diff.1.len());

    // gather the transaction diff and apply on utxo database
    let mut add: Vec<(Transaction, H256)> = vec![];
    let mut remove: Vec<(Transaction, H256)> = vec![];
    for hash in diff.0 {
        let block = blockdb.get(&hash).unwrap().unwrap();
        let content = match block.content {
            Content::Transaction(data) => data,
            _ => unreachable!(),
        };
        let mut transactions = content.transactions.iter().map(|t| (t.clone(), t.hash())).collect();
        // TODO: precompute the hash here. Note that although lazy-eval for tx hash, and we could have 
        // just called hash() here without storing the results (the results will be cached in the struct),
        // such function call will be optimized away by LLVM. As a result, we have to manually pass the hash
        // here. The same for added transactions below. This is a very ugly hack.
        add.append(&mut transactions);
    }
    for hash in diff.1 {
        let block = blockdb.get(&hash).unwrap().unwrap();
        let content = match block.content {
            Content::Transaction(data) => data,
            _ => unreachable!(),
        };
        let mut transactions = content.transactions.iter().map(|t| (t.clone(), t.hash())).collect();
        remove.append(&mut transactions);
    }
    return (add, remove);
}

pub fn update_utxo(add: &[(Transaction, H256)], remove: &[(Transaction, H256)], utxodb: &UtxoDatabase) -> (Vec<Input>, Vec<Input>) 
{
    let coin_diff = utxodb.apply_diff(&add, &remove).unwrap();
    return coin_diff;
}

pub fn update_wallet(add: &[Input], remove: &[Input], wallet: &Wallet)
{
    wallet.apply_diff(&add, &remove).unwrap();
}
