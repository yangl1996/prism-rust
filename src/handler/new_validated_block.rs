use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use crate::miner::memory_pool::MemoryPool;
use crate::utxodb::UtxoDatabase;
use crate::network::message;
use crate::network::server::Handle as ServerHandle;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use std::sync::Mutex;

pub fn new_validated_block(
    block: Block,
    mempool: &Mutex<MemoryPool>,
    blockdb: &BlockDatabase,
    chain: &BlockChain,
    server: &ServerHandle,
    utxodb: &UtxoDatabase,
    wallet: &Wallet,
) {
    // insert the new block into the blockdb
    blockdb.insert(&block).unwrap();

    // if this block is a transaction, remove transactions from mempool
    match &block.content {
        Content::Transaction(content) => {
            let mut mempool = mempool.lock().unwrap();
            for tx in &content.transactions {
                mempool.remove_by_hash(&tx.hash());
                // the inputs have been used here, so remove all transactions in the mempool that
                // tries to use the input again.
                for input in tx.input.iter() {
                    mempool.remove_by_input(input);
                }
            }
            drop(mempool);
        }
        _ => (),
    };

    // insert the new block into the blockchain
    let diff = chain.insert_block(&block).unwrap();
    drop(chain);

    // gather the transaction diff and apply on utxo database
    let mut add: Vec<Transaction> = vec![];
    let mut remove: Vec<Transaction> = vec![];
    for hash in diff.0 {
        let block = blockdb.get(&hash).unwrap().unwrap();
        let content = match block.content {
            Content::Transaction(data) => data,
            _=> unreachable!(),
        };
        let mut transactions = content.transactions.clone();
        add.append(&mut transactions);
    }
    for hash in diff.1 {
        let block = blockdb.get(&hash).unwrap().unwrap();
        let content = match block.content {
            Content::Transaction(data) => data,
            _=> unreachable!(),
        };
        let mut transactions = content.transactions.clone();
        remove.append(&mut transactions);
    }

    let coin_diff = utxodb.apply_diff(&add, &remove).unwrap();
    wallet.update(&coin_diff.0, &coin_diff.1).unwrap();


    // tell the neighbors that we have a new block
    server.broadcast(message::Message::NewBlockHashes(vec![block.hash()]));
}
