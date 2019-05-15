use crate::block::proposer::genesis as proposer_genesis;
use crate::block::voter::genesis as voter_genesis;
use crate::block::Block;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use bincode::{deserialize, serialize};
use rocksdb::{self, Options, DB, ColumnFamilyDescriptor};
use std::sync::atomic::{AtomicU64, Ordering};

const BLOCK_CF: &str = "BLOCK";
const BLOCK_ARRIVAL_ORDER_CF: &str = "BLOCK_ARRIVAL_ORDER";

/// Database that stores blocks.
pub struct BlockDatabase {
    /// The underlying RocksDB handle.
    db: rocksdb::DB,
    /// The number of blocks in this database.
    count: AtomicU64,
}

impl BlockDatabase {
    /// Open the database at the given path, and create a new one if missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let block_cf = ColumnFamilyDescriptor::new(BLOCK_CF, Options::default());
        let block_arrival_order_cf = ColumnFamilyDescriptor::new(BLOCK_ARRIVAL_ORDER_CF, Options::default());
        let cfs = vec![block_cf, block_arrival_order_cf];
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        return Ok(BlockDatabase {
            db: db,
            count: AtomicU64::new(0),
        });
    }

    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;

        let block_cf = db.db.cf_handle(BLOCK_CF).unwrap();
        let block_arrival_order_cf = db.db.cf_handle(BLOCK_ARRIVAL_ORDER_CF).unwrap();

        let mut counter: u64 = 0;
        // insert proposer genesis block
        let proposer_genesis_hash_u8: [u8; 32] = (*PROPOSER_GENESIS_HASH).into();
        db.db.put_cf(
            block_cf,
            &(*PROPOSER_GENESIS_HASH),
            &serialize(&proposer_genesis()).unwrap(),
        )?;
        db.db.put_cf(
            block_arrival_order_cf,
            &counter.to_ne_bytes(),
            &(*PROPOSER_GENESIS_HASH)
        )?;
        counter += 1;

        // insert voter genesis blocks
        for i in 0..NUM_VOTER_CHAINS {
            db.db.put_cf(
                block_cf,
                &VOTER_GENESIS_HASHES[i as usize],
                &serialize(&voter_genesis(i as u16)).unwrap(),
            )?;
            db.db.put_cf(
                block_arrival_order_cf,
                &counter.to_ne_bytes(),
                &VOTER_GENESIS_HASHES[i as usize],
                )?;
            counter += 1;
        }

        db.count
            .store(counter, Ordering::Relaxed);
        return Ok(db);
    }

    /// Load database from a given path
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let db = Self::open(&path)?;
        return Ok(db);
    }

    /// Insert a new block to the database and returns the sequence number of the block.
    pub fn insert(&self, block: &Block) -> Result<u64, rocksdb::Error> {
        let block_cf = self.db.cf_handle(BLOCK_CF).unwrap();
        let block_arrival_order_cf = self.db.cf_handle(BLOCK_ARRIVAL_ORDER_CF).unwrap();
        let hash: H256 = block.hash();
        let serialized = serialize(block).unwrap();
        let counter = self.count.fetch_add(1, Ordering::Relaxed);
        self.db.put_cf(block_cf, &hash, &serialized)?;
        self.db.put_cf(block_arrival_order_cf, &counter.to_ne_bytes(), &hash)?;
        return Ok(counter);
    }

    /// Get a block from the database.
    pub fn get(&self, hash: &H256) -> Result<Option<Block>, rocksdb::Error> {
        let block_cf = self.db.cf_handle(BLOCK_CF).unwrap();
        let block_arrival_order_cf = self.db.cf_handle(BLOCK_ARRIVAL_ORDER_CF).unwrap();
        let serialized = self.db.get_cf(block_cf, hash)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
        }
    }

    /// Get the number of blocks in the database.
    pub fn num_blocks(&self) -> u64 {
        let count = self.count.load(Ordering::Relaxed);
        return count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_and_get() {
        let db = BlockDatabase::new(&std::path::Path::new(
            "/tmp/blockdb_tests_insert_get_and_delete.rocksdb",
        ))
        .unwrap();
        let block = proposer_genesis();
        let seq = db.insert(&block).unwrap();
        let got = db.get(&block.hash()).unwrap().unwrap();
        let num_block = db.num_blocks();
        assert_eq!(got.hash(), block.hash());
        assert_eq!(num_block, 1 + NUM_VOTER_CHAINS as u64 + 1);
        assert_eq!(seq, num_block - 1);
    }
}
