use super::{CoinId, UTXODatabase, UTXO};
use crate::crypto::hash::Hashable;
use crate::transaction::generator as tx_generator;
use crate::transaction::Transaction;
use crate::state::CoinData;

// Converts a transaction to bunch of utxos. TODO: Where should we put this function?
pub fn tx_to_utxos(transaction: &Transaction) -> Vec<UTXO> {
    let mut utxos: Vec<UTXO> = vec![];
    for (index, output) in transaction.output.iter().enumerate() {
        let coin_id = CoinId {
            hash: transaction.hash(),
            index: index as u32,
        };
        let coin_data = CoinData {
            value: output.value,
            recipient: output.recipient,
        };
        let utxo = UTXO {
            coin_id,
            coin_data,
        };
        utxos.push(utxo);
    }
    return utxos;
}

pub fn random() -> UTXODatabase {
    //1. init database
    let default_path = "/tmp/random_state_db_rocksdb";
    let statedb_path = std::path::Path::new(&default_path);
    let statedb = UTXODatabase::new(statedb_path).unwrap();

    //2. generate random txs
    for _ in 0..20 {
        let transaction = tx_generator::random();
        for utxo in tx_to_utxos(&transaction) {
            statedb.insert(&utxo);
        }
    }
    return statedb;
}
