use crate::block::proposer::genesis as proposer_genesis;
use crate::block::voter::genesis as voter_genesis;
use crate::block::Block;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use bincode::{deserialize, serialize};
use std::sync::Mutex;

/// Database that stores blocks.
pub struct BlockDatabase {
    /// The underlying RocksDB handle.
    handle: rocksdb::DB,
    /// The number of blocks in this database.
    count: Mutex<u64>,
}

impl BlockDatabase {
    /// Create a new database at the given path.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let db_handle = rocksdb::DB::open_default(path)?;

        // insert proposer genesis block
        let proposer_genesis_hash_u8: [u8; 32] = (*PROPOSER_GENESIS_HASH).into();
        db_handle.put(
            &proposer_genesis_hash_u8,
            &serialize(&proposer_genesis()).unwrap(),
        )?;

        // insert voter genesis blocks
        for i in 0..NUM_VOTER_CHAINS {
            let voter_genesis_hash_u8: [u8; 32] = VOTER_GENESIS_HASHES[i as usize].into();
            db_handle.put(
                &voter_genesis_hash_u8,
                &serialize(&voter_genesis(i as u16)).unwrap(),
            )?;
        }

        return Ok(BlockDatabase {
            handle: db_handle,
            // TODO: what about the genesis blocks we just inserted?
            count: Mutex::new(0),
        });
    }

    /// Insert a new block to the database.
    pub fn insert(&self, block: &Block) -> Result<(), rocksdb::Error> {
        let hash_u8: [u8; 32] = block.hash().into();
        let serialized = serialize(block).unwrap();
        let mut count = self.count.lock().unwrap();
        *count += 1;
        return self.handle.put(&hash_u8, &serialized);
    }

    /// Get a block from the database.
    pub fn get(&self, hash: H256) -> Result<Option<Block>, rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        let serialized = self.handle.get(&hash_u8)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
        }
    }

    /// Delete a block from the database
    // TODO: what if the block does not exist?
    pub fn delete(&self, hash: H256) -> Result<(), rocksdb::Error> {
        let hash_u8: [u8; 32] = hash.into();
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        return self.handle.delete(&hash_u8);
    }

    /// Get the number of blocks in the database.
    pub fn num_blocks(&self) -> u64 {
        let count = self.count.lock().unwrap();
        return *count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::generator;
    use crate::crypto::hash::Hashable;

    /*
        #[test]
        fn insert_get_and_delete() {
            let db = BlockDatabase::new(&std::path::Path::new(
                "/tmp/blockdb_tests_insert_get_and_delete.rocksdb",
            ))
            .unwrap();
            let test_block = generator::tx_block();
            db.insert(&test_block).unwrap();
            let got = db.get(&test_block.hash()).unwrap().unwrap();
            let num_block = db.num_blocks();
            assert_eq!(got.hash(), test_block.hash());
            assert_eq!(num_block, 1);
            db.delete(&test_block.hash()).unwrap();
            let num_block = db.num_blocks();
            assert_eq!(db.get(&test_block.hash()).unwrap().is_none(), true);
            assert_eq!(num_block, 0);
        }
    */
}
