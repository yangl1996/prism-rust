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
use std::collections::HashSet;

pub struct LedgerManager {
    blockdb: Arc<BlockDatabase>,
    chain: Arc<BlockChain>,
    utxodb: Arc<UtxoDatabase>,
    wallet: Arc<Wallet>,
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

        // start thread that dispatches jobs to utxo manager
        let utxodb = Arc::clone(&self.utxodb);
        let (transaction_tx, transaction_rx) = mpsc::channel();
        let (notification_tx, notification_rx) = mpsc::channel();
        let (coin_diff_tx, coin_diff_rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                // get the diff
                let (mut added_tx, mut removed_tx) = tx_diff_rx.recv().unwrap();

                // dispatch transactions
                let count = added_tx.len() + removed_tx.len();
                for (t, h) in removed_tx.drain(..).rev() {
                    transaction_tx.send((false, t, h)).unwrap();
                }
                for (t, h) in added_tx.drain(..) {
                    transaction_tx.send((true, t, h)).unwrap();
                }

                // collect notification
                for _ in 0..count {
                    notification_rx.recv().unwrap();
                }
            }
        });

        // start utxo manager
        let utxo_manager = UtxoManager {
            utxodb: Arc::clone(&self.utxodb),
            transaction_chan: Arc::new(Mutex::new(transaction_rx)),
            coin_chan: coin_diff_tx,
            notification_chan: notification_tx,
        };
        utxo_manager.start(num_workers);

        // start thread that writes to wallet
        let wallet = Arc::clone(&self.wallet);
        thread::spawn(move || {
            loop {
                let coin_diff = coin_diff_rx.recv().unwrap();
                wallet.apply_diff(&coin_diff.0, &coin_diff.1);
            }
        });
    }
}

#[derive(Clone)]
struct UtxoManager {
    utxodb: Arc<UtxoDatabase>,
    /// Channel for dispatching jobs (add/delete, transaction, hash of transaction).
    transaction_chan: Arc<Mutex<mpsc::Receiver<(bool, Transaction, H256)>>>,
    /// Channel for returning added and removed coins.
    coin_chan: mpsc::Sender<(Vec<Input>, Vec<Input>)>,
    /// Channel for notifying the dispatcher about the completion of processing this transaction.
    notification_chan: mpsc::Sender<H256>,
}

impl UtxoManager {
    fn start(self, num_workers: usize) {
        for i in 0..num_workers {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let chan = self.transaction_chan.lock().unwrap();
            let (add, transaction, hash) = chan.recv().unwrap();
            drop(chan);
            if add {
                let diff = self.utxodb.add_transaction(&transaction, hash).unwrap();
                self.coin_chan.send(diff).unwrap();
            }
            else {
                let diff = self.utxodb.remove_transaction(&transaction, hash).unwrap();
                self.coin_chan.send(diff).unwrap();
            }
            self.notification_chan.send(hash).unwrap();
        }
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

