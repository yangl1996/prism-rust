use crate::transaction::{Input, Output, Transaction};
use bincode::{deserialize, serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::Mutex;

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

#[derive(Hash, Serialize)]
pub struct UTXO{
    transaction: Transaction, //TODO: We only need the hash of the tx and index.
    index: u32,
    value: u64
}


#[derive(Debug)]
pub struct UTXODatabase {
    handle: rocksdb::DB,
    count: Mutex<u64>,
}

impl UTXODatabase {
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let db_handle = rocksdb::DB::open_default(path)?;
        return Ok(UTXODatabase {
            handle: db_handle,
            count: Mutex::new(0),
        });
    }

    pub fn insert(&self, utxo: &UTXO) -> Result<()> {
        let mut hasher = DefaultHasher::new();
        let utxo_hash = serialize(& utxo.hash(&mut hasher)).unwrap();
        let utxo_serialized = serialize(utxo).unwrap();
        let mut count = self.count.lock().unwrap();
        *count += 1;
        return self.handle.put(&utxo_hash, &utxo_serialized);
    }

    pub fn delete(&mut self, utxo: &UTXO) -> Result<()> { //TODO: Should accept a hash of the utxo.
        let mut hasher = DefaultHasher::new();
        let utxo_hash = serialize(&utxo.hash(&mut hasher)).unwrap();
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        return self.handle.delete(utxo_hash);
    }

    //TODO: Should accept a hash of the utxo.
    //TODO: Only check if the utxo exists. It should not 'get' it.
    pub fn check(&self, utxo: &UTXO) -> Result<bool> {
        let mut hasher = DefaultHasher::new();
        let utxo_hash = serialize(&utxo.hash(&mut hasher)).unwrap();
        let serialized = self.handle.get(&utxo_hash)?;
        match serialized {
            None => return Ok(false),
            Some(s) => return Ok(true),
        }
    }

    pub fn num_utxo(&self) -> u64 {
        let count = self.count.lock().unwrap();
        return *count;
    }
}

// TODO: add tests
