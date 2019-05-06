use crate::crypto::hash::{Hashable, H256};
use crate::block::{Block, Content};
use crate::config::*;

use std::sync::Mutex;
use bincode::{deserialize, serialize};
use rocksdb::{ColumnFamily, Options, ColumnFamilyDescriptor, DB, WriteBatch};
use std::collections::{HashMap, HashSet, BTreeMap};

// Column family names for node/chain metadata
const PROPOSER_NODE_LEVEL_CF: &str = "PROPOSER_NODE_LEVEL";
const PROPOSER_NODE_STATUS_CF: &str = "PROPOSER_NODE_STATUS";
const VOTER_NODE_STATUS_CF: &str = "VOTER_NODE_STATUS";
const VOTER_NODE_LEVEL_CF: &str = "VOTER_NODE_LEVEL";
const VOTER_NODE_CHAIN_CF: &str = "VOTER_NODE_CHAIN";
const PROPOSER_TREE_LEVEL_CF: &str = "PROPOSER_TREE_LEVEL";
const VOTER_NODE_VOTED_LEVEL_CF: &str = "VOTER_NODE_VOTED_LEVEL";

// Column family names for graph neighbors
const PARENT_NEIGHBOR_CF: &str = "GRAPH_PARENT_NEIGHBOR";   // the proposer parent of a block
const VOTE_NEIGHBOR_CF: &str = "GRAPH_VOTE_NEIGHBOR";       // neighbors associated by a vote
const VOTER_PARENT_NEIGHBOR_CF: &str = "GRAPH_VOTER_PARENT_NEIGHBOR";   // the voter parent of a block
const TRANSACTION_REF_NEIGHBOR_CF: &str = "GRAPH_TRANSACTION_REF_NEIGHBOR";
const PROPOSER_REF_NEIGHBOR_CF: &str = "GRAPH_PROPOSER_REF_NEIGHBOR";

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

// cf_handle is a lightweight operation, it takes 44000 micro seconds to get 100000 cf handles

pub struct BlockChain {
    db: DB,
    proposer_best: Mutex<(H256, u64)>,
    voter_best: Vec<Mutex<(H256, u64)>>,
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
        let voter_node_chain_cf = ColumnFamilyDescriptor::new(VOTER_NODE_CHAIN_CF, Options::default());
        let voter_node_voted_level_cf = ColumnFamilyDescriptor::new(VOTER_NODE_VOTED_LEVEL_CF, Options::default());

        let mut proposer_tree_level_option = Options::default();
        proposer_tree_level_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let proposer_tree_level_cf = ColumnFamilyDescriptor::new(PROPOSER_TREE_LEVEL_CF, proposer_tree_level_option);

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
            voter_node_chain_cf,
            voter_node_voted_level_cf,
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
        let mut voter_best: Vec<Mutex<(H256, u64)>> = vec![];
        for _ in 0..NUM_VOTER_CHAINS {
            voter_best.push(Mutex::new((H256::default(), 0)));
        }
        
        let blockchain_db = Self {
            db: db,
            proposer_best: Mutex::new((H256::default(), 0)),
            voter_best: voter_best,
        };
        
        return Ok(blockchain_db);
    }
    
    /// Destroy the existing database at the given path, create a new one, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;
        // get cf handles
        let proposer_node_level_cf = db.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let proposer_node_status_cf = db.db.cf_handle(PROPOSER_NODE_STATUS_CF).unwrap();
        let voter_node_level_cf = db.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_status_cf = db.db.cf_handle(VOTER_NODE_STATUS_CF).unwrap();
        let voter_node_chain_cf = db.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = db.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = db.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = db.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let voter_parent_neighbor_cf = db.db.cf_handle(VOTER_PARENT_NEIGHBOR_CF).unwrap();
        let transaction_ref_neighbor_cf = db.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();
        let proposer_ref_neighbor_cf = db.db.cf_handle(PROPOSER_REF_NEIGHBOR_CF).unwrap();

        // insert genesis blocks
        let mut wb = WriteBatch::default();

        // proposer genesis block
        wb.put_cf(proposer_node_level_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(), 
                  serialize(&(0 as u64)).unwrap())?;
        wb.merge_cf(proposer_tree_level_cf, serialize(&(0 as u64)).unwrap(),
                    serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;
        let mut proposer_best = db.proposer_best.lock().unwrap();
        proposer_best.0 = *PROPOSER_GENESIS_HASH;
        drop(proposer_best);

        // voter genesis blocks
        for chain_num in 0..NUM_VOTER_CHAINS {
            wb.put_cf(parent_neighbor_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(), 
                      serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;
            wb.merge_cf(vote_neighbor_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(), 
                      serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;
            wb.merge_cf(vote_neighbor_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(), 
                      serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap())?;
            wb.put_cf(voter_node_level_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(), 
                      serialize(&(0 as u64)).unwrap())?;
            wb.put_cf(voter_node_voted_level_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(), 
                      serialize(&(0 as u64)).unwrap())?;
            wb.put_cf(voter_node_chain_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(), 
                      serialize(&(chain_num as u16)).unwrap())?;
            let mut voter_best = db.voter_best[chain_num as usize].lock().unwrap();
            voter_best.0 = VOTER_GENESIS_HASHES[chain_num as usize];
            drop(voter_best);
        }
        db.db.write(wb)?;

        return Ok(db);
    }

    pub fn insert_block(&self, block: &Block) -> Result<()> {
        // get cf handles
        let proposer_node_level_cf = self.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let proposer_node_status_cf = self.db.cf_handle(PROPOSER_NODE_STATUS_CF).unwrap();
        let voter_node_level_cf = self.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_status_cf = self.db.cf_handle(VOTER_NODE_STATUS_CF).unwrap();
        let voter_node_chain_cf = self.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = self.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = self.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = self.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let voter_parent_neighbor_cf = self.db.cf_handle(VOTER_PARENT_NEIGHBOR_CF).unwrap();
        let transaction_ref_neighbor_cf = self.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();
        let proposer_ref_neighbor_cf = self.db.cf_handle(PROPOSER_REF_NEIGHBOR_CF).unwrap();

        let mut wb = WriteBatch::default();

        // insert parent link
        let block_hash = block.hash();
        let parent_hash = block.header.parent;
        wb.put_cf(parent_neighbor_cf, serialize(&block_hash).unwrap(), 
                  serialize(&parent_hash).unwrap())?;

        match &block.content {
            Content::Proposer(content) => {
                // add ref'ed blocks
                wb.put_cf(proposer_ref_neighbor_cf, serialize(&block_hash).unwrap(),
                          serialize(&content.proposer_refs).unwrap())?;
                wb.put_cf(transaction_ref_neighbor_cf, serialize(&block_hash).unwrap(),
                          serialize(&content.transaction_refs).unwrap())?;
                // get current block level
                let parent_level: u64 = deserialize(&self.db.get_cf(proposer_node_level_cf,
                                                                   serialize(&parent_hash).unwrap())?
                                                    .unwrap()).unwrap();
                let self_level = parent_level + 1;
                // set current block level
                wb.put_cf(proposer_node_level_cf, serialize(&block_hash).unwrap(),
                          serialize(&self_level).unwrap())?;
                wb.merge_cf(proposer_tree_level_cf, serialize(&self_level).unwrap(),
                            serialize(&block_hash).unwrap())?;
                // set best block info
                let mut proposer_best = self.proposer_best.lock().unwrap();
                if self_level > proposer_best.1 {
                    proposer_best.0 = block_hash;
                    proposer_best.1 = self_level;
                }
                drop(proposer_best);
            }
            Content::Voter(content) => {
                // add voter parent
                let voter_parent_hash = content.voter_parent;
                wb.put_cf(voter_parent_neighbor_cf, serialize(&block_hash).unwrap(),
                          serialize(&content.voter_parent).unwrap())?;
                // get current block level and chain number
                let voter_parent_level: u64 = deserialize(&self.db.get_cf(voter_node_level_cf,
                                                          serialize(&voter_parent_hash).unwrap())?
                                              .unwrap()).unwrap();
                let voter_parent_chain: u16 = deserialize(&self.db.get_cf(voter_node_chain_cf,
                                                          serialize(&voter_parent_hash).unwrap())?
                                              .unwrap()).unwrap();
                let self_level = voter_parent_level + 1;
                let self_chain = voter_parent_chain;
                // set current block level and chain number
                wb.put_cf(voter_node_level_cf, serialize(&block_hash).unwrap(), 
                          serialize(&self_level).unwrap())?;
                wb.put_cf(voter_node_chain_cf, serialize(&block_hash).unwrap(), 
                          serialize(&self_chain).unwrap())?;
                // add voted blocks and set deepest voted level
                wb.put_cf(vote_neighbor_cf, serialize(&block_hash).unwrap(), 
                          serialize(&content.votes).unwrap())?;
                let mut deepest_voted_level = 0;
                for vote_hash in &content.votes {
                    wb.merge_cf(vote_neighbor_cf, serialize(&vote_hash).unwrap(), 
                                serialize(&block_hash).unwrap())?;
                    let voted_level: u64 = deserialize(&self.db.get_cf(proposer_node_level_cf,
                                                                       serialize(&vote_hash).unwrap())?
                                                       .unwrap()).unwrap();
                    if voted_level > deepest_voted_level {
                        deepest_voted_level = voted_level;
                    }
                }
                wb.put_cf(voter_node_voted_level_cf, serialize(&block_hash).unwrap(), 
                          serialize(&deepest_voted_level).unwrap())?;
                // set best block info
                let mut voter_best = self.voter_best[self_chain as usize].lock().unwrap();
                if self_level > voter_best.1 {
                    voter_best.0 = block_hash;
                    voter_best.1 = self_level;
                }
                drop(voter_best);
            }
            Content::Transaction(_) => {
            }
        }
        self.db.write(wb)?;
        return Ok(());
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
    use crate::block::{Block, Content, transaction, proposer, voter};
    use crate::crypto::hash::H256;
    use super::*;
    
    #[test]
    fn initialize_new() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_new.rocksdb").unwrap();
        // get cf handles
        let proposer_node_level_cf = db.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let proposer_node_status_cf = db.db.cf_handle(PROPOSER_NODE_STATUS_CF).unwrap();
        let voter_node_level_cf = db.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_status_cf = db.db.cf_handle(VOTER_NODE_STATUS_CF).unwrap();
        let voter_node_chain_cf = db.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = db.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = db.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = db.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let voter_parent_neighbor_cf = db.db.cf_handle(VOTER_PARENT_NEIGHBOR_CF).unwrap();
        let transaction_ref_neighbor_cf = db.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();
        let proposer_ref_neighbor_cf = db.db.cf_handle(PROPOSER_REF_NEIGHBOR_CF).unwrap();

        // validate proposer genesis
        let genesis_level: u64 = deserialize(&db.db.get_cf(proposer_node_level_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(genesis_level, 0);
        let level_0_blocks: Vec<H256> = deserialize(&db.db.get_cf(proposer_tree_level_cf, serialize(&(0 as u64)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level_0_blocks, vec![*PROPOSER_GENESIS_HASH]);
        assert_eq!(*db.proposer_best.lock().unwrap(), (*PROPOSER_GENESIS_HASH, 0));

        // validate voter genesis
        for chain_num in 0..NUM_VOTER_CHAINS {
            let genesis_level: u64 = deserialize(&db.db.get_cf(voter_node_level_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap()).unwrap().unwrap()).unwrap();
            assert_eq!(genesis_level, 0);
            let voted_level: u64 = deserialize(&db.db.get_cf(voter_node_voted_level_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap()).unwrap().unwrap()).unwrap();
            assert_eq!(voted_level, 0);
            let genesis_chain: u16 = deserialize(&db.db.get_cf(voter_node_chain_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap()).unwrap().unwrap()).unwrap();
            assert_eq!(genesis_chain, chain_num as u16);
            let parent: H256 = deserialize(&db.db.get_cf(parent_neighbor_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap()).unwrap().unwrap()).unwrap();
            assert_eq!(parent, *PROPOSER_GENESIS_HASH);
            let voted_proposer: Vec<H256> = deserialize(&db.db.get_cf(vote_neighbor_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap()).unwrap().unwrap()).unwrap();
            assert_eq!(voted_proposer, vec![*PROPOSER_GENESIS_HASH]);
            assert_eq!(*db.voter_best[chain_num as usize].lock().unwrap(), (VOTER_GENESIS_HASHES[chain_num as usize], 0));
        }
        let voters: Vec<H256> = deserialize(&db.db.get_cf(vote_neighbor_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(voters, *VOTER_GENESIS_HASHES);
    }

    #[test]
    fn insert_block() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_insert_block.rocksdb").unwrap();
        // get cf handles
        let proposer_node_level_cf = db.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let proposer_node_status_cf = db.db.cf_handle(PROPOSER_NODE_STATUS_CF).unwrap();
        let voter_node_level_cf = db.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_status_cf = db.db.cf_handle(VOTER_NODE_STATUS_CF).unwrap();
        let voter_node_chain_cf = db.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = db.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = db.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = db.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let voter_parent_neighbor_cf = db.db.cf_handle(VOTER_PARENT_NEIGHBOR_CF).unwrap();
        let transaction_ref_neighbor_cf = db.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();
        let proposer_ref_neighbor_cf = db.db.cf_handle(PROPOSER_REF_NEIGHBOR_CF).unwrap();

        // Create a transaction block on the proposer genesis.
        let new_transaction_content = Content::Transaction(transaction::Content::new(vec![]));
        let new_transaction_block = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_transaction_content,
            [0; 32],
            H256::default()
        );
        db.insert_block(&new_transaction_block).unwrap();

        let parent: H256 = deserialize(&db.db.get_cf(parent_neighbor_cf, serialize(&new_transaction_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(parent, *PROPOSER_GENESIS_HASH);

        // Create two proposer blocks, both attached to the genesis proposer block. The first one
        // refers to nothing, and the second one refers to the first one and the transaction block
        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block_1 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [0; 32],
            H256::default()
        );
        db.insert_block(&new_proposer_block_1).unwrap();

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![new_transaction_block.hash()], vec![new_proposer_block_1.hash()]));
        let new_proposer_block_2 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [0; 32],
            H256::default()
        );
        db.insert_block(&new_proposer_block_2).unwrap();

        let parent: H256 = deserialize(&db.db.get_cf(parent_neighbor_cf, serialize(&new_proposer_block_2.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(parent, *PROPOSER_GENESIS_HASH);
        let level: u64 = deserialize(&db.db.get_cf(proposer_node_level_cf, serialize(&new_proposer_block_2.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level, 1);
        let level_1_blocks: Vec<H256> = deserialize(&db.db.get_cf(proposer_tree_level_cf, serialize(&(1 as u64)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level_1_blocks, vec![new_proposer_block_1.hash(), new_proposer_block_2.hash()]);
        let proposer_ref: Vec<H256> = deserialize(&db.db.get_cf(proposer_ref_neighbor_cf, serialize(&new_proposer_block_2.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(proposer_ref, vec![new_proposer_block_1.hash()]);
        let transaction_ref: Vec<H256> = deserialize(&db.db.get_cf(transaction_ref_neighbor_cf, serialize(&new_proposer_block_2.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(transaction_ref, vec![new_transaction_block.hash()]);
        assert_eq!(*db.proposer_best.lock().unwrap(), (new_proposer_block_1.hash(), 1));

        // Create a voter block attached to proposer block 2 and the first voter chain, and vote for proposer block 1.
        let new_voter_content = Content::Voter(voter::Content::new(1, VOTER_GENESIS_HASHES[0], vec![new_proposer_block_1.hash()]));
        let new_voter_block = Block::new(
            new_proposer_block_2.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [0; 32],
            H256::default()
        );
        db.insert_block(&new_voter_block).unwrap();

        let parent: H256 = deserialize(&db.db.get_cf(parent_neighbor_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(parent, new_proposer_block_2.hash());
        let voter_parent: H256 = deserialize(&db.db.get_cf(voter_parent_neighbor_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(voter_parent, VOTER_GENESIS_HASHES[0]);
        let level: u64 = deserialize(&db.db.get_cf(voter_node_level_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level, 1);
        let voted_level: u64 = deserialize(&db.db.get_cf(voter_node_voted_level_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level, 1);
        let voted: Vec<H256> = deserialize(&db.db.get_cf(vote_neighbor_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(voted, vec![new_proposer_block_1.hash()]);
        let voter: Vec<H256> = deserialize(&db.db.get_cf(vote_neighbor_cf, serialize(&new_proposer_block_1.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(voter, vec![new_voter_block.hash()]);
        assert_eq!(*db.voter_best[0].lock().unwrap(), (new_voter_block.hash(), 1));

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

