use crate::crypto::hash::H256;
use crate::transaction::{Input, Output, Transaction};
use bincode::{deserialize, serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

// Bitcoin UTXO is much more complicated because they have extra seg-wit and locktime.


type CoinId = Input;

#[derive(Hash, Serialize)]
pub struct UTXO {
    coin_id: CoinId, // Hash of the transaction. This along with the index is the coin index is the key.
    value: u64,
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

    /// Add utxo to t he
    pub fn insert(&self, utxo: &UTXO) -> Result<()> {
        let key = serialize(&utxo.coin_id).unwrap();
        let value = serialize(&utxo.value).unwrap();
        let mut count = self.count.lock().unwrap();
        *count += 1;
        return self.handle.put(&key, &value);
    }

    pub fn delete(&mut self, coin_id: CoinId) -> Result<()> {
        let key = serialize(&coin_id).unwrap();
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        return self.handle.delete(key);
    }

    pub fn check(&mut self, coin_id: CoinId) -> Result<bool> {
        let key = serialize(&coin_id).unwrap();
        let serialized = self.handle.get(&key)?;
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
