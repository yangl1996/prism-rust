use crate::block::Block;
use crate::crypto::hash::H256;
use bincode::{deserialize, serialize};
use std::sync::Mutex;

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

pub struct BlockDatabase {
    handle: rocksdb::DB,
    count: Mutex<u64>,
}

impl BlockDatabase {
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let db_handle = rocksdb::DB::open_default(path)?;
        return Ok(BlockDatabase {
            handle: db_handle,
            count: Mutex::new(0),
        });
    }

    pub fn insert(&self, hash: &H256, block: &Block) -> Result<()> {
        let hash_u8: [u8; 32] = hash.into(); // TODO: implement H256 asref<[u8]>
        let serialized = serialize(block).unwrap();
        let mut count = self.count.lock().unwrap();
        *count += 1;
        return self.handle.put(&hash_u8, &serialized);
    }

    pub fn get(&self, hash: &H256) -> Result<Option<Block>> {
        let hash_u8: [u8; 32] = hash.into();
        let serialized = self.handle.get(&hash_u8)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
        }
    }

    pub fn delete(&self, hash: &H256) -> Result<()> {
        let hash_u8: [u8; 32] = hash.into();
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        return self.handle.delete(&hash_u8);
    }

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

    #[test]
    fn insert_get_and_delete() {
        let db = BlockDatabase::new(&std::path::Path::new(
            "/tmp/blockdb_tests_insert_get_and_delete.rocksdb",
        ))
        .unwrap();
        let test_block = generator::tx_block();
        db.insert(&test_block.hash(), &test_block).unwrap();
        let got = db.get(&test_block.hash()).unwrap().unwrap();
        let num_block = db.num_blocks();
        assert_eq!(got.hash(), test_block.hash());
        assert_eq!(num_block, 1);
        db.delete(&test_block.hash()).unwrap();
        let num_block = db.num_blocks();
        assert_eq!(db.get(&test_block.hash()).unwrap().is_none(), true);
        assert_eq!(num_block, 0);
    }
}
