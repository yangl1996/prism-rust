use crate::crypto::hash::{Hashable, H256};
use serde::{Serialize, Deserialize};
use bincode::{deserialize, serialize};
use std::sync::Mutex;
use rocksdb::ColumnFamily;

/// Database that stores blockchain.
pub struct BlockChainDatabase {
    /// The underlying RocksDB handle.
    handle: rocksdb::DB,
}

impl BlockChainDatabase {
    /// Create a new database at the given path.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let db_handle = rocksdb::DB::open_default(path)?;
        return Ok(BlockChainDatabase {
            handle: db_handle,
        });
    }

    /// Insert into the database.
    pub fn insert<D: Serialize>(&self, cf: ColumnFamily, hash: &H256, data: D) -> Result<(), rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        let serialized_data = serialize(&data).unwrap();
        return self.handle.put_cf(cf, &hash_u8, &serialized_data);
    }

    /// Get data from the database.
    pub fn get<D: Deserialize>(&self, cf: ColumnFamily, hash: H256) -> Result<Option<D>, rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        let serialized = self.handle.get_cf(cf, &hash_u8)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
        }
    }

    pub fn delete(&self, cf: ColumnFamily, hash: H256) -> Result<(), rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        return self.handle.delete_cf(cf, &hash_u8);
    }
}

#[cfg(test)]
mod tests {

}
