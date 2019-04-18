use super::{CoinId, UTXODatabase, UTXO};
use crate::crypto::hash::Hashable;
use crate::state::CoinData;
use crate::transaction::generator as tx_generator;
use crate::transaction::Transaction;
use rand::{Rng, RngCore};
// tx_to_utxos moved to handler?

pub fn random() -> UTXODatabase {
    //1. init database
    let mut rng = rand::thread_rng();
    let statedb_path = format!("/tmp/random_state_db_rocksdb_{}", rng.next_u32());
    let statedb_path = std::path::Path::new(&statedb_path);
    let statedb = UTXODatabase::new(statedb_path).unwrap();

    return statedb;
}