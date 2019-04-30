use crate::crypto::hash::{Hashable, H256};
use bincode::{deserialize, serialize};
use rocksdb::ColumnFamily;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Database that stores blockchain.
pub struct BlockChainDatabase {
    /// The underlying RocksDB handle.
    pub handle: rocksdb::DB,
}

impl BlockChainDatabase {
    /// Create a new database at the given path.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let db_handle = rocksdb::DB::open_default(path)?;
        return Ok(BlockChainDatabase { handle: db_handle });
    }

    /// Insert into the database.
    pub fn insert<D: Serialize>(
        &self,
        cf_name: &str,
        hash: &H256,
        data: D,
    ) -> Result<(), rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        let cf = self.handle.cf_handle(cf_name).unwrap();
        let serialized_data = serialize(&data).unwrap();
        return self.handle.put_cf(cf, &hash_u8, &serialized_data);
    }

    //    /// Get data from the database.
    //    pub fn get<D: Deserialize>(&self, cf_name: &str, hash: &H256) -> Result<Option<D>, rocksdb::Error> {
    //        let hash_u8: [u8; 32] = hash.into();
    //        let cf = self.handle.cf_handle(cf_name).unwrap();
    //        let serialized = self.handle.get_cf(cf, &hash_u8)?;
    //        match serialized {
    //            None => return Ok(None),
    //            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
    //        }
    //    }

    //TODO: Check the key without getting the value (Use Bloom filters maybe?)
    pub fn check(&self, cf_name: &str, hash: &H256) -> Result<bool, rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        let cf = self.handle.cf_handle(cf_name).unwrap();
        let serialized = self.handle.get_cf(cf, &hash_u8)?;
        match serialized {
            None => return Ok(false),
            Some(_s) => return Ok(true),
        }
    }

    pub fn delete(&self, cf_name: &str, hash: &H256) -> Result<(), rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        let cf = self.handle.cf_handle(cf_name).unwrap();
        return self.handle.delete_cf(cf, &hash_u8);
    }
}

#[cfg(test)]
mod tests {}
