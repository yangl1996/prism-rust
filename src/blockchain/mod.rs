use crate::crypto::hash::{Hashable, H256};
use crate::block::{Block, Content};
use crate::config::*;

use std::sync::Mutex;
use bincode::{deserialize, serialize};
use rocksdb::{ColumnFamily, Options, ColumnFamilyDescriptor, DB, WriteBatch};
use std::collections::{HashMap, HashSet, BTreeMap};

use std::iter::FromIterator;

// Column family names for node/chain metadata
const PROPOSER_NODE_LEVEL_CF: &str = "PROPOSER_NODE_LEVEL";
const PROPOSER_NODE_STATUS_CF: &str = "PROPOSER_NODE_STATUS";
const VOTER_NODE_STATUS_CF: &str = "VOTER_NODE_STATUS";
const VOTER_NODE_LEVEL_CF: &str = "VOTER_NODE_LEVEL";
const PROPOSER_TREE_LEVEL_CF: &str = "PROPOSER_TREE_LEVEL";

// Column family names for graph neighbors
const PARENT_NEIGHBOR_CF: &str = "GRAPH_PARENT_NEIGHBOR";   // the proposer parent of a block
const VOTE_NEIGHBOR_CF: &str = "GRAPH_VOTE_NEIGHBOR";       // neighbors associated by a vote
const VOTER_PARENT_NEIGHBOR_CF: &str = "GRAPH_VOTER_PARENT_NEIGHBOR";   // the voter parent of a block
const TRANSACTION_REF_NEIGHBOR_CF: &str = "GRAPH_TRANSACTION_REF_NEIGHBOR";
const PROPOSER_REF_NEIGHBOR_CF: &str = "GRAPH_PROPOSER_REF_NEIGHBOR";

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

// TODO: to maintain consistency within the blockchain db, we don't create a hierarchy of strcuts.
// Instead, the BlockChain struct has every cf handle and db handle, and batch writes into big ops,
// such as "add a new block"
pub struct BlockChain {
    db: DB,
}

// Functions to edit the blockchain
impl BlockChain {
    /// Open the blockchain database at the given path, and create missing column families.
    /// This function also populates the metadata fields with default values, and those
    /// fields must be initialized later.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let proposer_node_level_cf = ColumnFamilyDescriptor::new(PROPOSER_NODE_LEVEL_CF, Options::default());
        let proposer_node_status_cf = ColumnFamilyDescriptor::new(PROPOSER_NODE_STATUS_CF, Options::default());
        let voter_node_level_cf = ColumnFamilyDescriptor::new(VOTER_NODE_LEVEL_CF, Options::default());
        let voter_node_status_cf = ColumnFamilyDescriptor::new(VOTER_NODE_STATUS_CF, Options::default());
        let proposer_tree_level_cf = ColumnFamilyDescriptor::new(PROPOSER_TREE_LEVEL_CF, Options::default());

        let mut parent_neighbor_option = Options::default();
        parent_neighbor_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let parent_neighbor_cf = ColumnFamilyDescriptor::new(PARENT_NEIGHBOR_CF, parent_neighbor_option);

        let mut vote_neighbor_option = Options::default();
        vote_neighbor_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let vote_neighbor_cf = ColumnFamilyDescriptor::new(VOTE_NEIGHBOR_CF, vote_neighbor_option);

        let mut voter_parent_neighbor_option = Options::default();
        voter_parent_neighbor_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let voter_parent_neighbor_cf = ColumnFamilyDescriptor::new(VOTER_PARENT_NEIGHBOR_CF, voter_parent_neighbor_option);

        let mut transaction_ref_neighbor_option = Options::default();
        transaction_ref_neighbor_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let transaction_ref_neighbor_cf = ColumnFamilyDescriptor::new(TRANSACTION_REF_NEIGHBOR_CF, transaction_ref_neighbor_option);

        let mut proposer_ref_neighbor_option = Options::default();
        proposer_ref_neighbor_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let proposer_ref_neighbor_cf = ColumnFamilyDescriptor::new(PROPOSER_REF_NEIGHBOR_CF, proposer_ref_neighbor_option);


        let cfs = vec![
            proposer_node_level_cf,
            proposer_node_status_cf,
            voter_node_level_cf,
            voter_node_status_cf,
            proposer_tree_level_cf,
            parent_neighbor_cf,
            vote_neighbor_cf,
            voter_parent_neighbor_cf,
            transaction_ref_neighbor_cf,
            proposer_ref_neighbor_cf,
        ];
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        
        let blockchain_db = Self {
            db: db,
        };
        
        return Ok(blockchain_db);
    }
    
    /// Destroy the existing database at the given path, create a new one, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        DB::destroy(&Options::default(), &path)?; // TODO: handle error
        let db = Self::open(&path)?;
        return Ok(db);
    }
}

fn h256_vec_append_merge(_: &[u8], existing_val: Option<&[u8]>, operands: &mut rocksdb::merge_operator::MergeOperands) -> Option<Vec<u8>> {
    let mut existing: Vec<H256> = match existing_val {
        Some(v) => deserialize(v).unwrap(),
        None => vec![],
    };
    for op in operands {
        let new_hash: H256 = deserialize(op).unwrap();
        existing.push(new_hash);
    }
    let result: Vec<u8> = serialize(&existing).unwrap();
    return Some(result);
}

#[cfg(test)]
mod tests {
    use crate::block::Block;
    use crate::crypto::hash::H256;
    use super::*;
    
    #[test]
    fn initialize_new() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_new.rocksdb").unwrap();
    }

    #[test]
    fn merge_operator_counter() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_merge_op_counter.rocksdb").unwrap();
        let cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();

        // merge with an nonexistent entry
        db.db.merge_cf(cf, b"testkey", serialize(&H256::default()).unwrap());
        let result: Vec<H256> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![H256::default()]);

        // merge with an existing entry
        db.db.merge_cf(cf, b"testkey", serialize(&H256::default()).unwrap());
        db.db.merge_cf(cf, b"testkey", serialize(&H256::default()).unwrap());
        let result: Vec<H256> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![H256::default(), H256::default(), H256::default()]);
    }
}

