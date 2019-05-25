pub mod transaction_generator;

use crate::utxodb::UtxoDatabase;
use std::sync::Arc;
use crate::transaction::{CoinId, Input, Output, Transaction};
use crate::wallet::Wallet;
use crate::crypto::hash::H256;


/// Gives 100 coins of 100 worth to every given address.
pub fn ico(
    recipients: &[H256], // addresses of all the ico recipients
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
) -> Result<(), rocksdb::Error> {
    let funding = Transaction {
        input: vec![],
        output: recipients
            .iter()
            .map(|recipient| {
                (0..50000).map(move |_| Output {
                    value: 100,
                    recipient: recipient.clone(),
                })
            })
            .flatten()
            .collect(),
        authorization: vec![],
    };
    let diff = utxodb.apply_diff(&[funding], &[]).unwrap();
    wallet.apply_diff(&diff.0, &diff.1).unwrap();
    Ok(())
}
