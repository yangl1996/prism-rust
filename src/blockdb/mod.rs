use crate::block::Block;
use crate::crypto::hash::H256;
use bincode::{serialize, deserialize};

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

pub struct BlockDatabase {
    handle: rocksdb::DB,
}

impl BlockDatabase {
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let db_handle = rocksdb::DB::open_default(path)?;
        return Ok(BlockDatabase {
            handle: db_handle,
        });
    }

    pub fn insert(&self, hash: &H256, block: &Block) -> Result<()> {
        let hash_u8: [u8; 32] = hash.into();    // TODO: implement H256 asref<[u8]>
        let serialized = serialize(block).unwrap();
        return self.handle.put(&hash_u8, &serialized);
    }

    pub fn get(&self, hash: &H256) -> Result<Option<Block>> {
        let hash_u8: [u8; 32] = hash.into();
        let serialized = self.handle.get(&hash_u8)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap()))
        }
    }

    pub fn delete(&self, hash: &H256) -> Result<()> {
        let hash_u8: [u8; 32] = hash.into();
        return self.handle.delete(&hash_u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::generator;
    use crate::crypto::hash::Hashable;
    
    #[test]
    fn insert_get_and_delete() {
        let db = BlockDatabase::new(&std::path::Path::new("/tmp/prismdb.rocksdb")).unwrap();
        let test_block = generator::tx_block();
        db.insert(&test_block.hash(), &test_block);
        let got = db.get(&test_block.hash()).unwrap().unwrap();
        assert_eq!(got.hash(), test_block.hash());
        db.delete(&test_block.hash());
        assert_eq!(db.get(&test_block.hash()).unwrap().is_none(), true);
    }
}
