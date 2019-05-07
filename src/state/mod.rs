use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Input, CoinId};
use bincode::{deserialize, serialize};
use rocksdb::{DB, Options, WriteBatch, ColumnFamilyDescriptor};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::config::*;

const PROPOSER_LEADER_SEQUENCE_CF: &str = "PROPOSER_LEADER_SEQUENCE";   // level (u64) to hash
const PROPOSER_VOTE_LEVEL_CF: &str = "PROPOSER_VOTE_LEVEL";   // hash to Vec<level(u64)>

pub struct UtxoDatabase {
    db: rocksdb::DB,
    /// The level of each vote on the unconfirmed proposer blocks.
    unconfirmed_proposer_vote_level: Mutex<HashMap<H256, Vec<u64>>>,
    /// The hashes of proposer blocks at each unconfirmed level.
    unconfirmed_proposer_level: Mutex<HashMap<u64, Vec<H256>>>,
    /// The hash and depth of the best block of each voter chain.
    voter_best: Vec<Mutex<(H256, u64)>>,
}

impl UtxoDatabase {
    /// Open the database at the given path, and create a new one if one is missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let proposer_leader_sequence_cf = ColumnFamilyDescriptor::new(PROPOSER_LEADER_SEQUENCE_CF, Options::default());
        let mut proposer_vote_level_option = Options::default();
        proposer_vote_level_option.set_merge_operator("append u64 vec", u64_vec_append_merge, None);
        let proposer_vote_level_cf = ColumnFamilyDescriptor::new(PROPOSER_VOTE_LEVEL_CF, proposer_vote_level_option);

        let cfs = vec![
            proposer_leader_sequence_cf,
            proposer_vote_level_cf,
        ];
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let mut voter_best: Vec<Mutex<(H256, u64)>> = vec![];
        for _ in 0..NUM_VOTER_CHAINS {
            voter_best.push(Mutex::new((H256::default(), 0)));
        }

        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        return Ok(UtxoDatabase {
            db: db,
            unconfirmed_proposer_vote_level: Mutex::new(HashMap::new()),
            unconfirmed_proposer_level: Mutex::new(HashMap::new()),
            voter_best: voter_best,
        });
    }
    
    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;

        // get cf handles
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_vote_level_cf = db.db.cf_handle(PROPOSER_VOTE_LEVEL_CF).unwrap();

        // mark the proposer genesis block as the confirmed leader of level 0
        let mut wb = WriteBatch::default();
        wb.put_cf(proposer_leader_sequence_cf, serialize(&(0 as u64)).unwrap(), serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;

        // mark the voter genesis blocks as the best block of each chain, and record vote levels
        // for the proposer genesis block
        for chain_num in 0..NUM_VOTER_CHAINS {
            let mut voter_best = db.voter_best[chain_num as usize].lock().unwrap();
            voter_best.0 = VOTER_GENESIS_HASHES[chain_num as usize];
            wb.merge_cf(proposer_vote_level_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(), serialize(&(0 as u64)).unwrap())?;
        }
        db.db.write(wb)?;
        return Ok(db);
    }

    /*
    /// Apply the vote diff, specified as a collection of added votes and removed votes and their
    /// depth.
    pub fn apply_vote_diff(&self, added: Vec<(H256, u64)>, removed: Vec<(H256, u64)>) {
        let mut proposer_vote_level = self.unconfirmed_proposer_vote_level.lock().unwrap();
        let mut proposer_level = self.unconfirmed_proposer_level.lock().unwrap();
        
        // deal with deletion
        
    }
    */
}

fn u64_vec_append_merge(_: &[u8], existing_val: Option<&[u8]>, operands: &mut rocksdb::merge_operator::MergeOperands) -> Option<Vec<u8>> {
    let mut existing: Vec<u64> = match existing_val {
        Some(v) => deserialize(v).unwrap(),
        None => vec![],
    };
    for op in operands {
        let new_elem: u64 = deserialize(op).unwrap();
        existing.push(new_elem);
    }
    let result: Vec<u8> = serialize(&existing).unwrap();
    return Some(result);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initialize_new() {
        let db = UtxoDatabase::new("/tmp/prism_test_utxodatabase_new.rocksdb").unwrap();
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_vote_level_cf = db.db.cf_handle(PROPOSER_VOTE_LEVEL_CF).unwrap();

        let level_0_leader: H256 = deserialize(&db.db.get_cf(proposer_leader_sequence_cf, serialize(&(0 as u64)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level_0_leader, *PROPOSER_GENESIS_HASH);

        let mut proposer_genesis_vote_level: Vec<u64> = vec![];
        for chain_num in 0..NUM_VOTER_CHAINS {
            let voter_best = db.voter_best[chain_num as usize].lock().unwrap();
            assert_eq!(*voter_best, (VOTER_GENESIS_HASHES[chain_num as usize], 0));
            proposer_genesis_vote_level.push(0);
        }
        let genesis_votes: Vec<u64> = deserialize(&db.db.get_cf(proposer_vote_level_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(genesis_votes, proposer_genesis_vote_level);
    }
    
    #[test]
    fn merge_operator() {
        let db = UtxoDatabase::new("/tmp/prism_test_utxodatabase_merge_op.rocksdb").unwrap();
        let cf = db.db.cf_handle(PROPOSER_VOTE_LEVEL_CF).unwrap();

        // merge with an nonexistent entry
        db.db.merge_cf(cf, b"testkey", serialize(&(1 as u64)).unwrap()).unwrap();
        let result: Vec<u64> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![1]);

        // merge with an existing entry
        db.db.merge_cf(cf, b"testkey", serialize(&(2 as u64)).unwrap()).unwrap();
        db.db.merge_cf(cf, b"testkey", serialize(&(3 as u64)).unwrap()).unwrap();
        let result: Vec<u64> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }
}
