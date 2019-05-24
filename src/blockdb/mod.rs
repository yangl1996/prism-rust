use crate::block::proposer::genesis as proposer_genesis;
use crate::block::voter::genesis as voter_genesis;
use crate::block::Block;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use bincode::{deserialize, serialize};
use rocksdb::{self, ColumnFamilyDescriptor, Options, DB};
use std::convert::TryInto;
use std::sync::atomic::{AtomicU64, Ordering};

const BLOCK_CF: &str = "BLOCK";
const BLOCK_ARRIVAL_ORDER_CF: &str = "BLOCK_ARRIVAL_ORDER";
const BLOCK_SEQUENCE_NUMBER_CF: &str = "BLOCK_SEQUENCE_NUMBER";

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
        let block_arrival_order_cf =
            ColumnFamilyDescriptor::new(BLOCK_ARRIVAL_ORDER_CF, Options::default());
        let block_sequence_number_cf =
            ColumnFamilyDescriptor::new(BLOCK_SEQUENCE_NUMBER_CF, Options::default());
        let cfs = vec![block_cf, block_arrival_order_cf, block_sequence_number_cf];
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
        let block_sequence_number_cf = db.db.cf_handle(BLOCK_SEQUENCE_NUMBER_CF).unwrap();

        let mut counter: u64 = 0;
        // insert proposer genesis block
        db.db.put_cf(
            block_cf,
            &(*PROPOSER_GENESIS_HASH),
            &serialize(&proposer_genesis()).unwrap(),
        )?;
        db.db.put_cf(
            block_arrival_order_cf,
            &counter.to_ne_bytes(),
            &(*PROPOSER_GENESIS_HASH),
        )?;
        db.db.put_cf(
            block_sequence_number_cf,
            &(*PROPOSER_GENESIS_HASH),
            &counter.to_ne_bytes(),
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
            db.db.put_cf(
                block_sequence_number_cf,
                &VOTER_GENESIS_HASHES[i as usize],
                &counter.to_ne_bytes(),
            )?;
            counter += 1;
        }

        db.count.store(counter, Ordering::Relaxed);
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
        let block_sequence_number_cf = self.db.cf_handle(BLOCK_SEQUENCE_NUMBER_CF).unwrap();
        let hash: H256 = block.hash();
        let serialized = serialize(block).unwrap();
        let counter = self.count.fetch_add(1, Ordering::Relaxed);
        self.db.put_cf(block_cf, &hash, &serialized)?;
        self.db
            .put_cf(block_arrival_order_cf, &counter.to_ne_bytes(), &hash)?;
        self.db
            .put_cf(block_sequence_number_cf, &hash, &counter.to_ne_bytes())?;
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

    pub fn blocks_after(&self, after: &H256, batch_size: u64) -> BlocksInArrivalOrder {
        let block_sequence_number_cf = self.db.cf_handle(BLOCK_SEQUENCE_NUMBER_CF).unwrap();
        let start_seq = u64::from_ne_bytes(
            self.db
                .get_cf(block_sequence_number_cf, &after)
                .unwrap()
                .unwrap()[0..8]
                .try_into()
                .unwrap(),
        ) + 1;
        return BlocksInArrivalOrder {
            seq: start_seq,
            batch: batch_size,
            db: &self,
        };
    }

    /// Get the number of blocks in the database.
    pub fn num_blocks(&self) -> u64 {
        let count = self.count.load(Ordering::Relaxed);
        return count;
    }

    /// Get the hash of the latest block.
    pub fn latest_block_hash(&self) -> Result<H256, rocksdb::Error> {
        let block_arrival_order_cf = self.db.cf_handle(BLOCK_ARRIVAL_ORDER_CF).unwrap();
        let mut count = self.count.load(Ordering::Relaxed) - 1;
        // TODO: this is a hack to deal with a potential race condition: counter is increased
        // before the hash for that value is committed into the database.
        loop {
            let hash_serialized = self
                .db
                .get_cf(block_arrival_order_cf, &count.to_ne_bytes())?;
            let hash: H256 = match hash_serialized {
                Some(v) => {
                    let bytes: [u8; 32] = (&v[0..32]).try_into().unwrap();
                    return Ok(bytes.into());
                }
                None => {
                    count -= 1;
                    continue;
                }
            };
        }
    }
}

pub struct BlocksInArrivalOrder<'a> {
    seq: u64,
    batch: u64,
    db: &'a BlockDatabase,
}

impl<'a> std::iter::Iterator for BlocksInArrivalOrder<'a> {
    type Item = Vec<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        let block_cf = self.db.db.cf_handle(BLOCK_CF).unwrap();
        let block_arrival_order_cf = self.db.db.cf_handle(BLOCK_ARRIVAL_ORDER_CF).unwrap();
        let num_blocks = self.db.count.load(Ordering::Relaxed);
        let mut this_batch: u64 = 0;
        let mut result: Vec<Block> = vec![];
        while self.seq < num_blocks && this_batch < self.batch {
            let hash_bytes = self
                .db
                .db
                .get_cf(block_arrival_order_cf, &self.seq.to_ne_bytes())
                .unwrap()
                .unwrap();
            let block: Block =
                deserialize(&self.db.db.get_cf(block_cf, &hash_bytes).unwrap().unwrap()).unwrap();
            result.push(block);
            self.seq += 1;
            this_batch += 1;
        }
        if result.is_empty() {
            return None;
        } else {
            return Some(result);
        }
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

    #[test]
    fn blocks_after() {
        let db = BlockDatabase::new(&std::path::Path::new(
            "/tmp/blockdb_tests_blocks_by_arrival_order.rocksdb",
        ))
        .unwrap();
        // try to get all blocks after the proposer genesis
        let iter = db.blocks_after(&(*PROPOSER_GENESIS_HASH), 2);
        let mut next_voter = 0;
        for batch in iter {
            if next_voter + 1 < NUM_VOTER_CHAINS {
                assert_eq!(batch[0].hash(), voter_genesis(next_voter).hash());
                assert_eq!(batch[1].hash(), voter_genesis(next_voter + 1).hash());
                next_voter += 2;
            } else {
                assert_eq!(batch[0].hash(), voter_genesis(next_voter).hash());
                next_voter += 1;
            }
        }
        assert_eq!(next_voter as u16, NUM_VOTER_CHAINS as u16);
    }

    #[test]
    fn latest_block_hash() {
        let db = BlockDatabase::new(&std::path::Path::new(
            "/tmp/blockdb_tests_latest_block_hash.rocksdb",
        ))
        .unwrap();
        assert_eq!(
            db.latest_block_hash().unwrap(),
            VOTER_GENESIS_HASHES[NUM_VOTER_CHAINS as usize - 1]
        );
    }
}
