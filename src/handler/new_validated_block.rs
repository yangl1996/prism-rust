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

pub fn new_validated_block(
    block: &Block,
    mempool: &Mutex<MemoryPool>,
    blockdb: &BlockDatabase,
    chain: &BlockChain,
    server: &ServerHandle,
    utxodb: &UtxoDatabase,
    wallet: &Wallet,
) {
    PERFORMANCE_COUNTER.record_process_block(&block);
    // TODO: here mempool acts as a global lock. This is a dirty fix for data race in utxodb.
    let mut mempool = mempool.lock().unwrap();
    // insert the new block into the blockdb
    blockdb.insert(&block).unwrap();

    // if this block is a transaction, remove transactions from mempool
    match &block.content {
        Content::Transaction(content) => {
            for tx in &content.transactions {
                mempool.remove_by_hash(&tx.hash());
                // the inputs have been used here, so remove all transactions in the mempool that
                // tries to use the input again.
                for input in tx.input.iter() {
                    mempool.remove_by_input(input);
                }
            }
        }
        _ => (),
    };

    // insert the new block into the blockchain
    let diff = chain.insert_block(&block).unwrap();
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

    for transaction in &add {
        PERFORMANCE_COUNTER.record_confirm_transaction(&transaction);
    }
    for transaction in &remove {
        PERFORMANCE_COUNTER.record_deconfirm_transaction(&transaction);
    }

    let coin_diff = utxodb.apply_diff(&add, &remove).unwrap();
    wallet.apply_diff(&coin_diff.0, &coin_diff.1).unwrap();
    drop(mempool);

    // tell the neighbors that we have a new block
    server.broadcast(message::Message::NewBlockHashes(vec![block.hash()]));
}
