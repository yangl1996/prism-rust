use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Input, CoinId};
use bincode::{deserialize, serialize};
use rocksdb::{DB, Options, WriteBatch};
use crate::config::*;

pub struct UtxoDatabase {
    db: rocksdb::DB,
}

impl Ledger {
    /// Open the database at the given path, and create a new one if one is missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let cfs = vec![];
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        return Ok(Self{
            db: db,
        });
    }
    
    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;

        return Ok(db);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initialize_new() {
        let db = Ledger::new("/tmp/prism_test_ledger_new.rocksdb").unwrap();
    }
}
