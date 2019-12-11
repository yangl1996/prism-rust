use crate::block::proposer::genesis as proposer_genesis;
use crate::block::voter::genesis as voter_genesis;
use crate::block::{Block};
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use bincode::{deserialize, serialize};
use rocksdb::{self, ColumnFamilyDescriptor, Options, SliceTransform, DB};
use std::convert::TryInto;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::block::BlockType;

const BLOCK_CF: &str = "BLOCK";
const BLOCK_TYPE_CF: &str = "BLOCK_TYPE";

/// Database that stores blocks.
pub struct BlockDatabase {
    /// The underlying RocksDB handle.
    db: rocksdb::DB,
    /// The number of blocks in this database.
    count: AtomicU64,
}

impl BlockDatabase {
    /// Open the database at the given path, and create a new one if missing.
    fn open<P: AsRef<std::path::Path>>(
        path: P,
        _config: BlockchainConfig,
    ) -> Result<Self, rocksdb::Error> {
        let mut opts = Options::default();
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(32));
        opts.optimize_for_point_lookup(512);
        let block_cf = ColumnFamilyDescriptor::new(BLOCK_CF, opts);
        let mut opts = Options::default();
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(32));
        opts.optimize_for_point_lookup(512);
        let block_type_cf = ColumnFamilyDescriptor::new(BLOCK_TYPE_CF, opts);
        let cfs = vec![block_cf, block_type_cf];
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        Ok(BlockDatabase {
            db,
            count: AtomicU64::new(0),
        })
    }

    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(
        path: P,
        config: BlockchainConfig,
    ) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path, config.clone())?;

        let block_cf = db.db.cf_handle(BLOCK_CF).unwrap();
        let block_type_cf = db.db.cf_handle(BLOCK_TYPE_CF).unwrap();

        let mut counter: u64 = 0;
        // insert proposer genesis block
        db.db.put_cf(
            block_cf,
            &config.proposer_genesis,
            &serialize(&proposer_genesis()).unwrap(),
        )?;
        db.db.put_cf(
            block_type_cf,
            &config.proposer_genesis,
            &[BlockType::Proposer.into()],
        )?;
        counter += 1;

        // insert voter genesis blocks
        for i in 0..config.voter_chains {
            db.db.put_cf(
                block_cf,
                &config.voter_genesis[i as usize],
                &serialize(&voter_genesis(i as u16)).unwrap(),
            )?;
            db.db.put_cf(
                block_type_cf,
                &config.voter_genesis[i as usize],
                &[BlockType::Voter.into()],
                )?;
            counter += 1;
        }

        db.count.store(counter, Ordering::Relaxed);
        Ok(db)
    }

    /// Load database from a given path
    pub fn load<P: AsRef<std::path::Path>>(
        path: P,
        config: BlockchainConfig,
    ) -> Result<Self, rocksdb::Error> {
        let db = Self::open(&path, config)?;
        Ok(db)
    }

    /// Insert a new block to the database and returns the sequence number of the block.
    pub fn insert(&self, block: &Block) -> Result<u64, rocksdb::Error> {
        let block_cf = self.db.cf_handle(BLOCK_CF).unwrap();
        let block_type_cf = self.db.cf_handle(BLOCK_TYPE_CF).unwrap();
        let hash: H256 = block.hash();
        let serialized = serialize(block).unwrap();
        let counter = self.count.fetch_add(1, Ordering::Relaxed);
        self.db.put_cf(block_cf, &hash, &serialized)?;
        let block_type: u8 = block.block_type().into();
        self.db.put_cf(block_type_cf, &hash, &[block_type])?;
        Ok(counter)
    }

    pub fn insert_encoded(&self, hash: &H256, raw_block: &[u8], block_type: BlockType) -> Result<u64, rocksdb::Error> {
        let block_cf = self.db.cf_handle(BLOCK_CF).unwrap();
        let block_type_cf = self.db.cf_handle(BLOCK_TYPE_CF).unwrap();
        let counter = self.count.fetch_add(1, Ordering::Relaxed);
        self.db.put_cf(block_cf, &hash, &raw_block)?;
        self.db.put_cf(block_type_cf, &hash, &[block_type.into()])?;
        Ok(counter)
    }

    /// Get a block from the database.
    pub fn get(&self, hash: &H256) -> Result<Option<Block>, rocksdb::Error> {
        let block_cf = self.db.cf_handle(BLOCK_CF).unwrap();
        let serialized = self.db.get_pinned_cf(block_cf, hash)?;
        match serialized {
            None => Ok(None),
            Some(s) => Ok(Some(deserialize(&s).unwrap())),
        }
    }

    pub fn get_encoded(
        &self,
        hash: &H256,
    ) -> Result<Option<rocksdb::DBPinnableSlice>, rocksdb::Error> {
        let block_cf = self.db.cf_handle(BLOCK_CF).unwrap();
        let serialized = self.db.get_pinned_cf(block_cf, hash)?;
        Ok(serialized)
    }

    pub fn get_type(
        &self,
        hash: &H256,
    ) -> Result<Option<BlockType>, rocksdb::Error> {
        let blocktype_cf = self.db.cf_handle(BLOCK_TYPE_CF).unwrap();
        let serialized = self.db.get_pinned_cf(blocktype_cf, hash)?;
        match serialized {
            Some(d) => Ok(Some(d[0].try_into().unwrap())),
            None => Ok(None),
        }
    }

    pub fn contains(&self, hash: &H256) -> Result<bool, rocksdb::Error> {
        let block_cf = self.db.cf_handle(BLOCK_CF).unwrap();
        let serialized = self.db.get_pinned_cf(block_cf, hash)?;
        match serialized {
            None => Ok(false),
            Some(_) => Ok(true),
        }
    }

    /// Get the number of blocks in the database.
    pub fn num_blocks(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

