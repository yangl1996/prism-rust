use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Input, CoinId};
use bincode::{deserialize, serialize};
use rocksdb::{DB, Options, WriteBatch, ColumnFamilyDescriptor};
use crate::blockchain::BlockChain;
use std::collections::{HashSet};
use std::sync::{Mutex, Arc};
use crate::config::*;



pub struct Context {
    db: rocksdb::DB,
    unconfirmed_proposer: HashSet<H256>,
}

impl Ledger {
    /// Open the database at the given path, and create a new one if one is missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let proposer_leader_sequence_cf = ColumnFamilyDescriptor::new(PROPOSER_LEADER_SEQUENCE_CF, Options::default());
        let proposer_confirm_list_cf = ColumnFamilyDescriptor::new(PROPOSER_CONFIRM_LIST_CF, Options::default());

        let cfs = vec![
            proposer_leader_sequence_cf,
            proposer_confirm_list_cf,
        ];
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        return Ok(Self{
            db: db,
            unconfirmed_proposer: HashSet::new(),
        });
    }
    
    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;

        // get cf handles
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_confirm_list_cf = db.db.cf_handle(PROPOSER_CONFIRM_LIST_CF).unwrap();

        // mark the proposer genesis block as the confirmed leader of level 0
        let mut wb = WriteBatch::default();
        wb.put_cf(proposer_leader_sequence_cf, serialize(&(0 as u64)).unwrap(), serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;
        
        // mark the level 0 leader confirms no other blocks
        let level_0_confirms: Vec<H256> = vec![];
        wb.put_cf(proposer_confirm_list_cf, serialize(&(0 as u64)).unwrap(), serialize(&level_0_confirms).unwrap())?;

        db.db.write(wb)?;
        return Ok(db);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initialize_new() {
        let db = Ledger::new("/tmp/prism_test_ledger_new.rocksdb").unwrap();
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_confirm_list_cf = db.db.cf_handle(PROPOSER_CONFIRM_LIST_CF).unwrap();

        let level_0_leader: H256 = deserialize(&db.db.get_cf(proposer_leader_sequence_cf, serialize(&(0 as u64)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level_0_leader, *PROPOSER_GENESIS_HASH);

        let level_0_confirms: Vec<H256> = deserialize(&db.db.get_cf(proposer_confirm_list_cf,
                                                                    serialize(&(0 as u64)).unwrap())
                                                      .unwrap().unwrap()).unwrap();
        assert_eq!(level_0_confirms, vec![]);
    }
}
