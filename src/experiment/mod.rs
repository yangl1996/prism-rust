use std::cell::RefCell;
pub mod performance_counter;
pub mod transaction_generator;

use crate::block::proposer;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{CoinId, Input, Output, Transaction};
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use crate::config::DELTA;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn ico(
    recipients: &[H256], // addresses of all the ico recipients
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
    num_coins: usize,
    value: u64,
) -> Result<(), rocksdb::Error> {
    let recipients: Vec<H256> = recipients.to_vec();
    let recipients = Arc::new(Mutex::new(recipients));
    let mut ico_time = proposer::genesis().header.pos_metadata.timestamp;//SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    ico_time = ( ico_time / DELTA ) * DELTA ;
    // start a bunch of worker threads to commit those coins
    let mut workers = vec![];
    for _ in 0..16 {
        let recipients = Arc::clone(&recipients);
        let utxodb = Arc::clone(&utxodb);
        let wallet = Arc::clone(&wallet);
        let handle = thread::spawn(move || loop {
            let recipient = match recipients.lock().unwrap().pop() {
                Some(r) => r,
                None => break,
            };
            let tx = Transaction {
                input: vec![],
                output: (0..num_coins)
                    .map(move |_| Output {
                        value: value,
                        recipient: recipient,
                    })
                    .collect(),
                authorization: vec![],
                hash: RefCell::new(None),
            };
            let hash = tx.hash();
            let diff = utxodb.add_transaction(&tx, hash, ico_time).unwrap();
            wallet.apply_diff(&diff.0, &diff.1).unwrap();
        });
        workers.push(handle);
    }
    for child in workers.drain(..) {
        child.join();
    }
    utxodb.flush()?;
    Ok(())
}
