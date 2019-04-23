use super::UTXODatabase;
use rand::{RngCore};

pub fn random() -> UTXODatabase {
    //1. init database
    let mut rng = rand::thread_rng();
    let statedb_path = format!("/tmp/random_state_db_rocksdb_{}", rng.next_u32());
    let statedb_path = std::path::Path::new(&statedb_path);
    let statedb = UTXODatabase::new(statedb_path).unwrap();

    return statedb;
}
