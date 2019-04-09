
use crate::block::transaction::Content as TxContent;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Input, Output, Transaction};

use bincode::{deserialize, serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

pub type Result<T> = std::result::Result<T, rocksdb::Error>;
pub type CoinId = Input;

// Bitcoin UTXO is much more complicated because they have extra seg-wit and locktime.
#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct UTXO {
    pub coin_id: CoinId, // Hash of the transaction. This along with the index is the coin index is the key.
    pub value: u64,
}

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
        let key = serialize(&utxo.coin_id).unwrap();
        let value = serialize(&utxo.value).unwrap();
        let mut count = self.count.lock().unwrap();
        *count += 1;
        return self.handle.put(&key, &value);
    }

    pub fn delete(&mut self, coin_id: &CoinId) -> Result<()> {
        let key = serialize(coin_id).unwrap();
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        return self.handle.delete(key);
    }

    pub fn get(&self, coin_id: &CoinId) -> Result<Option<u64>> {
        let key = serialize(coin_id).unwrap();
        let serialized = self.handle.get(&key)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
        }
    }

    //TODO: Check the key without getting the value (Use Bloom filters maybe?)
    pub fn check(&mut self, coin_id: &CoinId) -> Result<bool> {
        let key = serialize(coin_id).unwrap();
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

