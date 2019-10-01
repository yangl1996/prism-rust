pub mod performance_counter;
pub mod transaction_generator;

use crate::crypto::hash::H256;
use crate::transaction::{CoinId, Output};
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use bincode::serialize;
use rocksdb::*;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn ico(
    recipients: &[H256], // addresses of all the ico recipients
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
    num_coins: usize,
    value: u64,
) -> Result<(), rocksdb::Error> {
    let recipients: Vec<(usize, H256)> = recipients.iter().copied().enumerate().collect();
    let recipients = Arc::new(Mutex::new(recipients));

    // start a bunch of worker threads to commit those coins
    let mut workers = vec![];
    for _ in 0..16 {
        let recipients = Arc::clone(&recipients);
        let utxodb = Arc::clone(&utxodb);
        let wallet = Arc::clone(&wallet);
        let handle = thread::spawn(move || {
            while let Some(recipient) = recipients.lock().unwrap().pop() {
                let transaction_id_start = (recipient.0 * num_coins) as u128;

                let mut write_opt = WriteOptions::default();
                write_opt.disable_wal(true);
                let output = Output {
                    value,
                    recipient: recipient.1,
                };
                let output_raw = serialize(&output).unwrap();
                for i in 0..num_coins {
                    let tx_uid = transaction_id_start + i as u128;
                    let tx_uid = tx_uid.to_ne_bytes();
                    let mut tx_hash_raw: [u8; 32] = [0; 32];
                    tx_hash_raw[16..32].copy_from_slice(&tx_uid);
                    let coinid = CoinId {
                        hash: tx_hash_raw.into(),
                        index: 0,
                    };
                    utxodb
                        .db
                        .put_opt(serialize(&coinid).unwrap(), output_raw.clone(), &write_opt)
                        .unwrap();
                    wallet.apply_diff(&[(coinid, output)], &[]).unwrap();
                }
            }
        });
        workers.push(handle);
    }
    for child in workers.drain(..) {
        child.join().unwrap();
    }
    utxodb.flush()?;
    Ok(())
}
