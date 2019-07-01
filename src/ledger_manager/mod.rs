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
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

pub struct LedgerManager {
    blockdb: Arc<BlockDatabase>,
    chain: Arc<BlockChain>,
    utxodb: Arc<UtxoDatabase>,
    wallet: Arc<Wallet>
}

impl LedgerManager {
    pub fn new(blockdb: &Arc<BlockDatabase>, chain: &Arc<BlockChain>, utxodb: &Arc<UtxoDatabase>, wallet: &Arc<Wallet>) -> Self {
        return Self {
            blockdb: Arc::clone(&blockdb),
            chain: Arc::clone(&chain),
            utxodb: Arc::clone(&utxodb),
            wallet: Arc::clone(&wallet),
        };
    }

    pub fn start(self, buffer_size: usize, num_workers: usize) {
        // start thread that updates transaction sequence
        let blockdb = Arc::clone(&self.blockdb);
        let chain = Arc::clone(&self.chain);
        let (tx_diff_tx, tx_diff_rx) = mpsc::sync_channel(buffer_size);
        thread::spawn(move || {
            loop {
                let tx_diff = update_transaction_sequence(&blockdb, &chain);
                tx_diff_tx.send(tx_diff).unwrap();
            }
        });

        // start thread that writes to utxo database
        let utxodb = Arc::clone(&self.utxodb);
        let (coin_diff_tx, coin_diff_rx) = mpsc::sync_channel(buffer_size);
        thread::spawn(move || {
            loop {
                let tx_diff = tx_diff_rx.recv().unwrap();
                let coin_diff = utxodb.apply_diff(&tx_diff.0, &tx_diff.1);
                coin_diff_tx.send(coin_diff).unwrap();
            }
        });

        // start thread that writes to wallet
        let wallet = Arc::clone(&self.wallet);
        thread::spawn(move || {
            loop {
                let coin_diff = coin_diff_rx.recv().unwrap().unwrap();
                wallet.apply_diff(&coin_diff.0, &coin_diff.1);
            }
        });
    }
}

fn update_transaction_sequence (
    blockdb: &BlockDatabase,
    chain: &BlockChain,
) -> (Vec<(Transaction, H256)>, Vec<(Transaction, H256)>) {
    let diff = chain.update_ledger().unwrap();
    PERFORMANCE_COUNTER.record_confirm_transaction_blocks(diff.0.len());
    PERFORMANCE_COUNTER.record_deconfirm_transaction_blocks(diff.1.len());

    // gather the transaction diff
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

