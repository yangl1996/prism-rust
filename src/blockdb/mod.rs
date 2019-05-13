use crate::block::proposer::genesis as proposer_genesis;
use crate::block::voter::genesis as voter_genesis;
use crate::block::Block;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use bincode::{deserialize, serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use rocksdb::{self, DB, Options};

/// Database that stores blocks.
pub struct BlockDatabase {
    /// The underlying RocksDB handle.
    handle: rocksdb::DB,
    /// The number of blocks in this database.
    count: AtomicUsize,
}

impl BlockDatabase {
    /// Open the database at the given path, and create a new one if missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        return Ok(BlockDatabase {
            handle: db,
            count: AtomicUsize::new(0),
        });
    }

    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;

        // insert proposer genesis block
        let proposer_genesis_hash_u8: [u8; 32] = (*PROPOSER_GENESIS_HASH).into();
        db.handle.put(
            &(*PROPOSER_GENESIS_HASH),
            &serialize(&proposer_genesis()).unwrap(),
        )?;

        // insert voter genesis blocks
        for i in 0..NUM_VOTER_CHAINS {
            db.handle.put(
                &VOTER_GENESIS_HASHES[i as usize],
                &serialize(&voter_genesis(i as u16)).unwrap(),
            )?;
        }

        db.count.store(1 + NUM_VOTER_CHAINS as usize, Ordering::Relaxed);
        return Ok(db);
    }

    /// Load database from a given path
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let db = Self::open(&path)?;
        return Ok(db);
    }

    /// Insert a new block to the database.
    pub fn insert(&self, block: &Block) -> Result<(), rocksdb::Error> {
        let hash: H256 = block.hash();
        let serialized = serialize(block).unwrap();
        self.count.fetch_add(1, Ordering::Relaxed);
        return self.handle.put(&hash, &serialized);
    }

    /// Get a block from the database.
    pub fn get(&self, hash: &H256) -> Result<Option<Block>, rocksdb::Error> {
        let serialized = self.handle.get(hash)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
        }
    }

    /// Delete a block from the database
    // TODO: what if the block does not exist?
    pub fn delete(&self, hash: &H256) -> Result<(), rocksdb::Error> {
        self.count.fetch_sub(1, Ordering::Relaxed);
        return self.handle.delete(hash);
    }

    /// Get the number of blocks in the database.
    pub fn num_blocks(&self) -> usize {
        let count = self.count.load(Ordering::Relaxed);
        return count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_get_and_delete() {
        let db = BlockDatabase::new(&std::path::Path::new(
            "/tmp/blockdb_tests_insert_get_and_delete.rocksdb",
        ))
        .unwrap();
        let block = proposer_genesis();
        db.insert(&block).unwrap();
        let got = db.get(&block.hash()).unwrap().unwrap();
        let num_block = db.num_blocks();
        assert_eq!(got.hash(), block.hash());
        assert_eq!(num_block, 1 + NUM_VOTER_CHAINS as usize + 1);
        db.delete(&block.hash()).unwrap();
        let num_block = db.num_blocks();
        assert_eq!(db.get(&block.hash()).unwrap().is_none(), true);
        assert_eq!(num_block, 1 + NUM_VOTER_CHAINS as usize);
    }
}
