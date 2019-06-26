use std::cell::RefCell;
pub mod transaction_generator;
pub mod performance_counter;

use crate::utxodb::UtxoDatabase;
use crate::transaction::{CoinId, Input, Output, Transaction};
use crate::wallet::Wallet;
use crate::crypto::hash::{H256, Hashable};

pub fn ico(
    recipients: &[H256], // addresses of all the ico recipients
    utxodb: &UtxoDatabase,
    wallet: &Wallet,
    num_coins: usize,
    value: u64,
) -> Result<(), rocksdb::Error> {
    let funding = Transaction {
        input: vec![],
        output: recipients
            .iter()
            .map(|recipient| {
                (0..num_coins).map(move |_| Output {
                    value: value,
                    recipient: recipient.clone(),
                })
            })
            .flatten()
            .collect(),
        authorization: vec![],
        hash: RefCell::new(None)
    };
    let hash = funding.hash();
    let diff = utxodb.apply_diff(&[(funding, hash)], &[]).unwrap();
    utxodb.flush()?;
    wallet.apply_diff(&diff.0, &diff.1).unwrap();
    Ok(())
}
