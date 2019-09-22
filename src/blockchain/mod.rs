use crate::block::{Block, Content};
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};

use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use bincode::{deserialize, serialize};
use log::{debug, info, trace};
use rocksdb::{ColumnFamilyDescriptor, Options, WriteBatch, DB};
use statrs::distribution::{Discrete, Poisson, Univariate};

use std::collections::{BTreeMap, HashMap, HashSet};

use std::ops::Range;
use std::sync::Mutex;


// Column family names for node/chain metadata
const PROPOSER_NODE_LEVEL_CF: &str = "PROPOSER_NODE_LEVEL"; // hash to node level (u64)
const VOTER_NODE_LEVEL_CF: &str = "VOTER_NODE_LEVEL"; // hash to node level (u64)
const VOTER_NODE_CHAIN_CF: &str = "VOTER_NODE_CHAIN"; // hash to chain number (u16)
const PROPOSER_TREE_LEVEL_CF: &str = "PROPOSER_TREE_LEVEL"; // level (u64) to hashes of blocks (Vec<hash>)
const VOTER_NODE_VOTED_LEVEL_CF: &str = "VOTER_NODE_VOTED_LEVEL"; // hash to max. voted level (u64)
const PROPOSER_NODE_VOTE_CF: &str = "PROPOSER_NODE_VOTE"; // hash to level and chain number of main chain votes (Vec<u16, u64>)
const PROPOSER_LEADER_SEQUENCE_CF: &str = "PROPOSER_LEADER_SEQUENCE"; // level (u64) to hash of leader block.
const PROPOSER_LEDGER_ORDER_CF: &str = "PROPOSER_LEDGER_ORDER"; // level (u64) to the list of proposer blocks confirmed
                                                                // by this level, including the leader itself. The list
                                                                // is in the order that those blocks should live in the ledger.
const PROPOSER_VOTE_COUNT_CF: &str = "PROPOSER_VOTE_COUNT"; // number of all votes on a block

// Column family names for graph neighbors
const PARENT_NEIGHBOR_CF: &str = "GRAPH_PARENT_NEIGHBOR"; // the proposer parent of a block
const VOTE_NEIGHBOR_CF: &str = "GRAPH_VOTE_NEIGHBOR"; // neighbors associated by a vote
const VOTER_PARENT_NEIGHBOR_CF: &str = "GRAPH_VOTER_PARENT_NEIGHBOR"; // the voter parent of a block
const TRANSACTION_REF_NEIGHBOR_CF: &str = "GRAPH_TRANSACTION_REF_NEIGHBOR";
const PROPOSER_REF_NEIGHBOR_CF: &str = "GRAPH_PROPOSER_REF_NEIGHBOR";

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

// cf_handle is a lightweight operation, it takes 44000 micro seconds to get 100000 cf handles

pub struct BlockChain {
    db: DB,
    proposer_best_level: Mutex<u64>,
    voter_best: Vec<Mutex<(H256, u64)>>,
    unreferred_transactions: Mutex<HashSet<H256>>,
    unreferred_proposers: Mutex<HashSet<H256>>,
    unconfirmed_proposers: Mutex<HashSet<H256>>,
    proposer_ledger_tip: Mutex<u64>,
    voter_ledger_tips: Mutex<Vec<H256>>,
}

// Functions to edit the blockchain
impl BlockChain {
    /// Open the blockchain database at the given path, and create missing column families.
    /// This function also populates the metadata fields with default values, and those
    /// fields must be initialized later.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let proposer_node_level_cf =
            ColumnFamilyDescriptor::new(PROPOSER_NODE_LEVEL_CF, Options::default());
        let voter_node_level_cf =
            ColumnFamilyDescriptor::new(VOTER_NODE_LEVEL_CF, Options::default());
        let voter_node_chain_cf =
            ColumnFamilyDescriptor::new(VOTER_NODE_CHAIN_CF, Options::default());
        let voter_node_voted_level_cf =
            ColumnFamilyDescriptor::new(VOTER_NODE_VOTED_LEVEL_CF, Options::default());
        let proposer_leader_sequence_cf =
            ColumnFamilyDescriptor::new(PROPOSER_LEADER_SEQUENCE_CF, Options::default());
        let proposer_ledger_order_cf =
            ColumnFamilyDescriptor::new(PROPOSER_LEDGER_ORDER_CF, Options::default());

        let mut proposer_tree_level_option = Options::default();
        proposer_tree_level_option.set_merge_operator(
            "append H256 vec",
            h256_vec_append_merge,
            None,
        );
        let proposer_tree_level_cf =
            ColumnFamilyDescriptor::new(PROPOSER_TREE_LEVEL_CF, proposer_tree_level_option);

        let mut proposer_node_vote_option = Options::default();
        proposer_node_vote_option.set_merge_operator("insert or remove vote", vote_vec_merge, None);
        let proposer_node_vote_cf =
            ColumnFamilyDescriptor::new(PROPOSER_NODE_VOTE_CF, proposer_node_vote_option);

        let mut parent_neighbor_option = Options::default();
        parent_neighbor_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let parent_neighbor_cf =
            ColumnFamilyDescriptor::new(PARENT_NEIGHBOR_CF, parent_neighbor_option);

        let mut vote_neighbor_option = Options::default();
        vote_neighbor_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let vote_neighbor_cf = ColumnFamilyDescriptor::new(VOTE_NEIGHBOR_CF, vote_neighbor_option);

        let mut proposer_vote_count_option = Options::default();
        proposer_vote_count_option.set_merge_operator("add to u64", u64_plus_merge, None);
        let proposer_vote_count_cf =
            ColumnFamilyDescriptor::new(PROPOSER_VOTE_COUNT_CF, proposer_vote_count_option);

        let mut voter_parent_neighbor_option = Options::default();
        voter_parent_neighbor_option.set_merge_operator(
            "append H256 vec",
            h256_vec_append_merge,
            None,
        );
        let voter_parent_neighbor_cf =
            ColumnFamilyDescriptor::new(VOTER_PARENT_NEIGHBOR_CF, voter_parent_neighbor_option);

        let mut transaction_ref_neighbor_option = Options::default();
        transaction_ref_neighbor_option.set_merge_operator(
            "append H256 vec",
            h256_vec_append_merge,
            None,
        );
        let transaction_ref_neighbor_cf = ColumnFamilyDescriptor::new(
            TRANSACTION_REF_NEIGHBOR_CF,
            transaction_ref_neighbor_option,
        );

        let mut proposer_ref_neighbor_option = Options::default();
        proposer_ref_neighbor_option.set_merge_operator(
            "append H256 vec",
            h256_vec_append_merge,
            None,
        );
        let proposer_ref_neighbor_cf =
            ColumnFamilyDescriptor::new(PROPOSER_REF_NEIGHBOR_CF, proposer_ref_neighbor_option);

        let cfs = vec![
            proposer_node_level_cf,
            voter_node_level_cf,
            voter_node_chain_cf,
            voter_node_voted_level_cf,
            proposer_leader_sequence_cf,
            proposer_ledger_order_cf,
            proposer_tree_level_cf,
            proposer_node_vote_cf,
            parent_neighbor_cf,
            vote_neighbor_cf,
            proposer_vote_count_cf,
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
            proposer_best_level: Mutex::new(0),
            voter_best: voter_best,
            unreferred_transactions: Mutex::new(HashSet::new()),
            unreferred_proposers: Mutex::new(HashSet::new()),
            unconfirmed_proposers: Mutex::new(HashSet::new()),
            proposer_ledger_tip: Mutex::new(0),
            voter_ledger_tips: Mutex::new(vec![H256::default(); NUM_VOTER_CHAINS as usize]),
        };

        return Ok(blockchain_db);
    }

    /// Destroy the existing database at the given path, create a new one, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;
        // get cf handles
        let proposer_node_level_cf = db.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let voter_node_level_cf = db.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_chain_cf = db.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = db.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_node_vote_cf = db.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();
        let proposer_tree_level_cf = db.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = db.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let proposer_vote_count_cf = db.db.cf_handle(PROPOSER_VOTE_COUNT_CF).unwrap();
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_ledger_order_cf = db.db.cf_handle(PROPOSER_LEDGER_ORDER_CF).unwrap();
        let proposer_ref_neighbor_cf = db.db.cf_handle(PROPOSER_REF_NEIGHBOR_CF).unwrap();
        let transaction_ref_neighbor_cf = db.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();

        // insert genesis blocks
        let mut wb = WriteBatch::default();

        // proposer genesis block
        wb.put_cf(
            proposer_node_level_cf,
            serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
            serialize(&(0 as u64)).unwrap(),
        )?;
        wb.merge_cf(
            proposer_tree_level_cf,
            serialize(&(0 as u64)).unwrap(),
            serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
        )?;
        let mut unreferred_proposers = db.unreferred_proposers.lock().unwrap();
        unreferred_proposers.insert(*PROPOSER_GENESIS_HASH);
        drop(unreferred_proposers);
        wb.put_cf(
            proposer_leader_sequence_cf,
            serialize(&(0 as u64)).unwrap(),
            serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
        )?;
        let proposer_genesis_ledger: Vec<H256> = vec![*PROPOSER_GENESIS_HASH];
        wb.put_cf(
            proposer_ledger_order_cf,
            serialize(&(0 as u64)).unwrap(),
            serialize(&proposer_genesis_ledger).unwrap(),
        )?;
        wb.put_cf(
            proposer_ref_neighbor_cf,
            serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
            serialize(&Vec::<H256>::new()).unwrap(),
        )?;
        wb.put_cf(
            transaction_ref_neighbor_cf,
            serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
            serialize(&Vec::<H256>::new()).unwrap(),
        )?;

        // voter genesis blocks
        let mut voter_ledger_tips = db.voter_ledger_tips.lock().unwrap();
        for chain_num in 0..NUM_VOTER_CHAINS {
            wb.put_cf(
                parent_neighbor_cf,
                serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
            )?;
            wb.merge_cf(
                vote_neighbor_cf,
                serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
            )?;
            wb.merge_cf(
                proposer_vote_count_cf,
                serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
                serialize(&(1 as u64)).unwrap(),
            )?;
            wb.merge_cf(
                proposer_node_vote_cf,
                serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
                serialize(&(true, chain_num as u16, 0 as u64)).unwrap(),
            )?;
            wb.put_cf(
                voter_node_level_cf,
                serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                serialize(&(0 as u64)).unwrap(),
            )?;
            wb.put_cf(
                voter_node_voted_level_cf,
                serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                serialize(&(0 as u64)).unwrap(),
            )?;
            wb.put_cf(
                voter_node_chain_cf,
                serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                serialize(&(chain_num as u16)).unwrap(),
            )?;
            let mut voter_best = db.voter_best[chain_num as usize].lock().unwrap();
            voter_best.0 = VOTER_GENESIS_HASHES[chain_num as usize];
            drop(voter_best);
            voter_ledger_tips[chain_num as usize] = VOTER_GENESIS_HASHES[chain_num as usize];
        }
        drop(voter_ledger_tips);
        db.db.write(wb)?;

        return Ok(db);
    }

    /// Insert a new block into the ledger. Returns the list of added transaction blocks and
    /// removed transaction blocks.
    pub fn insert_block(&self, block: &Block) -> Result<()> {
        // get cf handles
        let proposer_node_level_cf = self.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let voter_node_level_cf = self.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_chain_cf = self.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = self.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = self.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = self.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let proposer_vote_count_cf = self.db.cf_handle(PROPOSER_VOTE_COUNT_CF).unwrap();
        let voter_parent_neighbor_cf = self.db.cf_handle(VOTER_PARENT_NEIGHBOR_CF).unwrap();
        let transaction_ref_neighbor_cf = self.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();
        let proposer_ref_neighbor_cf = self.db.cf_handle(PROPOSER_REF_NEIGHBOR_CF).unwrap();

        let mut wb = WriteBatch::default();

        macro_rules! get_value {
            ($cf:expr, $key:expr) => {{
                deserialize(
                    &self
                        .db
                        .get_pinned_cf($cf, serialize(&$key).unwrap())?
                        .unwrap(),
                )
                .unwrap()
            }};
        }

        macro_rules! put_value {
            ($cf:expr, $key:expr, $value:expr) => {{
                wb.put_cf($cf, serialize(&$key).unwrap(), serialize(&$value).unwrap())?;
            }};
        }

        macro_rules! merge_value {
            ($cf:expr, $key:expr, $value:expr) => {{
                wb.merge_cf($cf, serialize(&$key).unwrap(), serialize(&$value).unwrap())?;
            }};
        }

        // insert parent link
        let block_hash = block.hash();
        let parent_hash = block.header.parent;
        put_value!(parent_neighbor_cf, block_hash, parent_hash);

        match &block.content {
            Content::Proposer(content) => {
                // add ref'ed blocks
                // note that the parent is the first proposer block that we refer
                let mut refed_proposer: Vec<H256> = vec![parent_hash];
                refed_proposer.extend(&content.proposer_refs);
                put_value!(proposer_ref_neighbor_cf, block_hash, refed_proposer);
                put_value!(
                    transaction_ref_neighbor_cf,
                    block_hash,
                    content.transaction_refs
                );
                // get current block level
                let parent_level: u64 = get_value!(proposer_node_level_cf, parent_hash);
                let self_level = parent_level + 1;
                // set current block level
                put_value!(proposer_node_level_cf, block_hash, self_level as u64);
                merge_value!(proposer_tree_level_cf, self_level, block_hash);

                // mark ourself as unreferred proposer
                // This should happen before committing to the database, since we want this
                // add operation to happen before later block deletes it. NOTE: we could do this
                // after committing to the database. The solution is to add a "pre-delete" set in
                // unreferred_proposers that collects the entries to delete before they are even
                // inserted. If we have this, we don't need to add/remove entries in order.
                let mut unreferred_proposers = self.unreferred_proposers.lock().unwrap();
                unreferred_proposers.insert(block_hash);
                drop(unreferred_proposers);

                // mark outself as unconfirmed proposer
                // This could happen before committing to database, since this block has to become
                // the leader or be referred by a leader. However, both requires the block to be
                // committed to the database. For the same reason, this should not happen after
                // committing to the database (think about the case where this block immediately
                // becomes the leader and is ready to be confirmed).
                let mut unconfirmed_proposers = self.unconfirmed_proposers.lock().unwrap();
                unconfirmed_proposers.insert(block_hash);
                drop(unconfirmed_proposers);

                // commit to the database and update proposer best in the same atomic operation
                // These two happen together to ensure that if a voter/proposer/transaction block
                // depend on this proposer block, the miner must have already known about this
                // proposer block and is using it as the proposer parent.
                let mut proposer_best = self.proposer_best_level.lock().unwrap();
                self.db.write(wb)?;
                if self_level > *proposer_best {
                    *proposer_best = self_level;
                    PERFORMANCE_COUNTER.record_update_proposer_main_chain(self_level as usize);
                }
                drop(proposer_best);

                // remove referenced proposer and transaction blocks from the unreferred list
                // This could happen after committing to the database. It's because that we are
                // only removing transaction blocks here, and the entries we are trying to remove
                // are guaranteed to be already there (since they are inserted before the
                // corresponding transaction blocks are committed).
                let mut unreferred_proposers = self.unreferred_proposers.lock().unwrap();
                for ref_hash in &content.proposer_refs {
                    unreferred_proposers.remove(&ref_hash);
                }
                unreferred_proposers.remove(&parent_hash);
                drop(unreferred_proposers);
                let mut unreferred_transactions = self.unreferred_transactions.lock().unwrap();
                for ref_hash in &content.transaction_refs {
                    unreferred_transactions.remove(&ref_hash);
                }
                drop(unreferred_transactions);

                info!(
                    "Adding proposer block {} at timestamp {} at level {}",
                    block_hash, block.header.timestamp, self_level
                );
            }
            Content::Voter(content) => {
                // add voter parent
                let voter_parent_hash = content.voter_parent;
                put_value!(voter_parent_neighbor_cf, block_hash, voter_parent_hash);
                // get current block level and chain number
                let voter_parent_level: u64 = get_value!(voter_node_level_cf, voter_parent_hash);
                let voter_parent_chain: u16 = get_value!(voter_node_chain_cf, voter_parent_hash);
                let self_level = voter_parent_level + 1;
                let self_chain = voter_parent_chain;
                // set current block level and chain number
                put_value!(voter_node_level_cf, block_hash, self_level as u64);
                put_value!(voter_node_chain_cf, block_hash, self_chain as u16);
                // add voting blocks for the proposer
                for proposer_hash in &content.votes {
                    merge_value!(proposer_vote_count_cf, proposer_hash, &(1 as u64));
                }
                // add voted blocks and set deepest voted level
                put_value!(vote_neighbor_cf, block_hash, content.votes);
                // set the voted level to be until proposer parent
                let proposer_parent_level: u64 = get_value!(proposer_node_level_cf, parent_hash);
                put_value!(
                    voter_node_voted_level_cf,
                    block_hash,
                    proposer_parent_level as u64
                );

                self.db.write(wb)?;

                // This should happen after writing to db, because other modules will follow
                // voter_best to query its metadata. We need to get the metadata into database
                // before we can "announce" this block to other modules. Also, this does not create
                // race condition, since this update is "stateless" - we are not append/removing
                // from a record.
                let mut voter_best = self.voter_best[self_chain as usize].lock().unwrap();
                // update best block
                if self_level > voter_best.1 {
                    PERFORMANCE_COUNTER
                        .record_update_voter_main_chain(voter_best.1 as usize, self_level as usize);
                    voter_best.0 = block_hash;
                    voter_best.1 = self_level;
                }
                drop(voter_best);
            }
            Content::Transaction(_content) => {
                // mark itself as unreferred
                // Note that this could happen before committing to db, because no module will try
                // to access transaction content based on pointers in unreferred_transactions.
                let mut unreferred_transactions = self.unreferred_transactions.lock().unwrap();
                unreferred_transactions.insert(block_hash);
                drop(unreferred_transactions);

                // This db write is only to facilitate check_existence
                self.db.write(wb)?;
            }
        }
        return Ok(());
    }

    pub fn update_ledger(&self) -> Result<(Vec<H256>, Vec<H256>)> {
        let proposer_node_vote_cf = self.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();
        let proposer_node_level_cf = self.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let proposer_leader_sequence_cf = self.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_ledger_order_cf = self.db.cf_handle(PROPOSER_LEDGER_ORDER_CF).unwrap();
        let proposer_ref_neighbor_cf = self.db.cf_handle(PROPOSER_REF_NEIGHBOR_CF).unwrap();
        let transaction_ref_neighbor_cf = self.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();

        macro_rules! get_value {
            ($cf:expr, $key:expr) => {{
                match self.db.get_pinned_cf($cf, serialize(&$key).unwrap())? {
                    Some(raw) => Some(deserialize(&raw).unwrap()),
                    None => None,
                }
            }};
        }

        // apply the vote diff while tracking the votes of which proposer levels are affected
        let mut wb = WriteBatch::default();
        macro_rules! merge_value {
            ($cf:expr, $key:expr, $value:expr) => {{
                wb.merge_cf($cf, serialize(&$key).unwrap(), serialize(&$value).unwrap())?;
            }};
        }

        let mut voter_ledger_tips = self.voter_ledger_tips.lock().unwrap();
        let mut affected_range: Range<u64> = Range {
            start: std::u64::MAX,
            end: std::u64::MIN,
        };

        for chain_num in 0..NUM_VOTER_CHAINS {
            // get the diff of votes on this voter chain
            let from = voter_ledger_tips[chain_num as usize];
            let voter_best = self.voter_best[chain_num as usize].lock().unwrap();
            let to = voter_best.0;
            drop(voter_best);
            voter_ledger_tips[chain_num as usize] = to;

            let (added, removed) = self.vote_diff(from, to)?;

            // apply the vote diff on the proposer main chain vote cf
            for vote in &removed {
                merge_value!(
                    proposer_node_vote_cf,
                    vote.0,
                    (false, chain_num as u16, vote.1)
                );
                let proposer_level: u64 = get_value!(proposer_node_level_cf, vote.0).unwrap();
                if proposer_level < affected_range.start {
                    affected_range.start = proposer_level;
                }
                if proposer_level >= affected_range.end {
                    affected_range.end = proposer_level + 1;
                }
            }

            for vote in &added {
                merge_value!(
                    proposer_node_vote_cf,
                    vote.0,
                    (true, chain_num as u16, vote.1)
                );
                let proposer_level: u64 = get_value!(proposer_node_level_cf, vote.0).unwrap();
                if proposer_level < affected_range.start {
                    affected_range.start = proposer_level;
                }
                if proposer_level >= affected_range.end {
                    affected_range.end = proposer_level + 1;
                }
            }
        }
        drop(voter_ledger_tips);
        // commit the votes into the database
        self.db.write(wb)?;

        // recompute the leader of each level that was affected
        let mut wb = WriteBatch::default();
        macro_rules! merge_value {
            ($cf:expr, $key:expr, $value:expr) => {{
                wb.merge_cf($cf, serialize(&$key).unwrap(), serialize(&$value).unwrap())?;
            }};
        }
        macro_rules! put_value {
            ($cf:expr, $key:expr, $value:expr) => {{
                wb.put_cf($cf, serialize(&$key).unwrap(), serialize(&$value).unwrap())?;
            }};
        }
        macro_rules! delete_value {
            ($cf:expr, $key:expr) => {{
                wb.delete_cf($cf, serialize(&$key).unwrap())?;
            }};
        }

        // we will recompute the leader starting from min. affected level or ledger tip + 1,
        // whichever is smaller. so make min. affected level the smaller of the two
        if affected_range.start < affected_range.end {
            let proposer_ledger_tip_lock = self.proposer_ledger_tip.lock().unwrap();
            let proposer_ledger_tip: u64 = *proposer_ledger_tip_lock;
            drop(proposer_ledger_tip_lock);
            if proposer_ledger_tip + 1 < affected_range.start {
                affected_range.start = proposer_ledger_tip + 1;
            }
        }

        // start actually recomputing the leaders
        let mut change_begin: Option<u64> = None;

        for level in affected_range {
            let proposer_blocks: Vec<H256> =
                get_value!(proposer_tree_level_cf, level as u64).unwrap();
            let existing_leader: Option<H256> =
                get_value!(proposer_leader_sequence_cf, level as u64);
            // compute the new leader of this level
            // we use the confirmation policy from https://arxiv.org/abs/1810.08092
            let new_leader: Option<H256> = {
                let mut new_leader: Option<H256> = None;

                // collect the depth of each vote on each proposer block
                let mut votes_depth: HashMap<&H256, Vec<u64>> = HashMap::new(); // chain number and vote depth casted on the proposer block
                let mut total_vote_count: u16 = 0;
                let mut total_vote_depth: u64 = 0;

                for block in &proposer_blocks {
                    let votes: Vec<(u16, u64)> = match get_value!(proposer_node_vote_cf, block) {
                        None => vec![],
                        Some(d) => d,
                    };
                    let mut vote_depth: Vec<u64> = vec![];
                    for (chain_num, vote_level) in &votes {
                        // TODO: cache the voter chain best levels
                        let voter_best = self.voter_best[*chain_num as usize].lock().unwrap();
                        let this_depth = voter_best.1 - vote_level + 1;
                        drop(voter_best);
                        total_vote_depth += this_depth;
                        total_vote_count += 1;
                        vote_depth.push(this_depth);
                    }
                    votes_depth.insert(block, vote_depth);
                }

                // For debugging purpose only. This is very important for security.
                // TODO: remove this check in the future
                if NUM_VOTER_CHAINS < total_vote_count {
                    panic!(
                        "NUM_VOTER_CHAINS: {} total_votes:{}",
                        NUM_VOTER_CHAINS, total_vote_count
                    )
                }

                // no point in going further if less than 3/5 votes are cast
                if total_vote_count > NUM_VOTER_CHAINS * 3 / 5 {
                    // calculate average of depth of the votes
                    let avg_vote_depth = total_vote_depth as f32 / total_vote_count as f32;
                    // expected voter depth of an adversary
                    let adversary_expected_vote_depth =
                        avg_vote_depth / (1.0 - ALPHA) / (1.0 - ADVERSARY_MINING_POWER)
                            * ADVERSARY_MINING_POWER;
                    let poisson = Poisson::new(adversary_expected_vote_depth as f64).unwrap();

                    // for each block calculate the lower bound on the number of votes
                    let mut votes_lcb: HashMap<&H256, f32> = HashMap::new();
                    let mut total_votes_lcb: f32 = 0.0;
                    let mut max_vote_lcb: f32 = 0.0;

                    for block in &proposer_blocks {
                        let votes = votes_depth.get(block).unwrap();

                        let mut block_votes_mean: f32 = 0.0; // mean E[X]
                        let mut block_votes_variance: f32 = 0.0; // Var[X]
                        let mut block_votes_lcb: f32 = 0.0;
                        for depth in votes.iter() {
                            // probability that the adversary will remove this vote
                            let mut p: f32 = 1.0 - poisson.cdf((*depth as f32 + 1.0).into()) as f32;
                            for k in 0..(*depth as u64) {
                                // probability that the adversary has mined k blocks
                                let p1 = poisson.pmf(k) as f32;
                                // probability that the adversary will overtake 'depth-k' blocks
                                let p2 = ((ADVERSARY_MINING_POWER)
                                    / (1.0 - ADVERSARY_MINING_POWER))
                                    .powi((depth - k + 1) as i32);
                                p += p1 * p2;
                            }
                            block_votes_mean += 1.0 - p;
                            block_votes_variance += p * (1.0 - p);
                        }
                        // using gaussian approximation
                        let tmp =
                            block_votes_mean - (block_votes_variance).sqrt() * (*QUANTILE_EPSILON);
                        if tmp > 0.0 {
                            block_votes_lcb += tmp;
                        }
                        votes_lcb.insert(block, block_votes_lcb);
                        total_votes_lcb += block_votes_lcb;

                        if max_vote_lcb < block_votes_lcb {
                            max_vote_lcb = block_votes_lcb;
                            new_leader = Some(*block);
                        }
                        // In case of a tie, choose block with lower hash.
                        if max_vote_lcb == block_votes_lcb && new_leader.is_some() {
                            // TODO: is_some required?
                            if *block < new_leader.unwrap() {
                                new_leader = Some(*block);
                            }
                        }
                    }
                    // check if the lcb_vote of new_leader is bigger than second best ucb votes
                    let remaining_votes = NUM_VOTER_CHAINS as f32 - total_votes_lcb;

                    // if max_vote_lcb is lesser than the remaining_votes, then a private block could
                    // get the remaining votes and become the leader block
                    if max_vote_lcb <= remaining_votes || new_leader.is_none() {
                        new_leader = None;
                    } else {
                        for p_block in &proposer_blocks {
                            // if the below condition is true, then final votes on p_block could overtake new_leader
                            if max_vote_lcb < votes_lcb.get(p_block).unwrap() + remaining_votes
                                && *p_block != new_leader.unwrap()
                            {
                                println!(
                                    "Candidate: {:?}, lb={}, second ub={}",
                                    new_leader,
                                    max_vote_lcb,
                                    votes_lcb.get(p_block).unwrap() + remaining_votes
                                );
                                new_leader = None;
                                break;
                            }
                            //In case of a tie, choose block with lower hash.
                            if max_vote_lcb == votes_lcb.get(p_block).unwrap() + remaining_votes
                                && *p_block < new_leader.unwrap()
                            {
                                new_leader = None;
                                break;
                            }
                        }
                    }
                }

                new_leader
            };

            if new_leader != existing_leader {
                info!("Confirming block at level {}", level);
                // mark it's the beginning of the change
                if change_begin.is_none() {
                    change_begin = Some(level);
                }
                match new_leader {
                    None => delete_value!(proposer_leader_sequence_cf, level as u64),
                    Some(new) => put_value!(proposer_leader_sequence_cf, level as u64, new),
                };
            }
        }
        // commit the new leaders into the database
        self.db.write(wb)?;

        // recompute the ledger from the first level whose leader changed
        if let Some(change_begin) = change_begin {
            let mut proposer_ledger_tip = self.proposer_ledger_tip.lock().unwrap();
            let mut unconfirmed_proposers = self.unconfirmed_proposers.lock().unwrap();
            let mut removed: Vec<H256> = vec![];
            let mut added: Vec<H256> = vec![];
            let mut wb = WriteBatch::default();
            macro_rules! merge_value {
                ($cf:expr, $key:expr, $value:expr) => {{
                    wb.merge_cf($cf, serialize(&$key).unwrap(), serialize(&$value).unwrap())?;
                }};
            }
            macro_rules! put_value {
                ($cf:expr, $key:expr, $value:expr) => {{
                    wb.put_cf($cf, serialize(&$key).unwrap(), serialize(&$value).unwrap())?;
                }};
            }
            macro_rules! delete_value {
                ($cf:expr, $key:expr) => {{
                    wb.delete_cf($cf, serialize(&$key).unwrap())?;
                }};
            }

            // deconfirm the blocks from change_begin all the way to previous ledger tip
            for level in change_begin..=*proposer_ledger_tip {
                let original_ledger: Vec<H256> =
                    get_value!(proposer_ledger_order_cf, level as u64).unwrap();
                delete_value!(proposer_ledger_order_cf, level as u64);
                for block in &original_ledger {
                    unconfirmed_proposers.insert(*block);
                    removed.push(*block);
                }
            }

            // recompute the ledger from change_begin until the first level where there's no leader
            // make sure that the ledger is continuous
            if change_begin <= *proposer_ledger_tip + 1 {
                for level in change_begin.. {
                    let leader: H256 = match get_value!(proposer_leader_sequence_cf, level as u64) {
                        None => {
                            *proposer_ledger_tip = level - 1;
                            break;
                        }
                        Some(leader) => leader,
                    };
                    // Get the sequence of blocks by doing a depth-first traverse
                    let mut order: Vec<H256> = vec![];
                    let mut stack: Vec<H256> = vec![leader];
                    while let Some(top) = stack.pop() {
                        // if it's already
                        // confirmed before, ignore it
                        if !unconfirmed_proposers.contains(&top) {
                            continue;
                        }
                        let refs: Vec<H256> = get_value!(proposer_ref_neighbor_cf, top).unwrap();

                        // add the current block to the ordered ledger, could be duplicated
                        order.push(top);

                        // search all referred blocks
                        for ref_hash in &refs {
                            stack.push(*ref_hash);
                        }
                    }

                    // reverse the order we just got
                    order.reverse();
                    // deduplicate, keep the one copy that is former in this order
                    order = order
                        .into_iter()
                        .filter(|h| unconfirmed_proposers.remove(h))
                        .collect();
                    put_value!(proposer_ledger_order_cf, level as u64, order);
                    added.extend(&order);
                }
            }
            // commit the new ledger into the database
            self.db.write(wb)?;

            let mut removed_transaction_blocks: Vec<H256> = vec![];
            let mut added_transaction_blocks: Vec<H256> = vec![];
            for block in &removed {
                let t: Vec<H256> = get_value!(transaction_ref_neighbor_cf, block).unwrap();
                removed_transaction_blocks.extend(&t);
            }
            for block in &added {
                let t: Vec<H256> = get_value!(transaction_ref_neighbor_cf, block).unwrap();
                added_transaction_blocks.extend(&t);
            }
            return Ok((added_transaction_blocks, removed_transaction_blocks));
        } else {
            return Ok((vec![], vec![]));
        }
    }

    /// Given two voter blocks on the same chain, calculate the added and removed votes when
    /// switching the main chain.
    fn vote_diff(&self, from: H256, to: H256) -> Result<(Vec<(H256, u64)>, Vec<(H256, u64)>)> {
        // get cf handles
        let voter_node_level_cf = self.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let vote_neighbor_cf = self.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let voter_parent_neighbor_cf = self.db.cf_handle(VOTER_PARENT_NEIGHBOR_CF).unwrap();

        macro_rules! get_value {
            ($cf:expr, $key:expr) => {{
                deserialize(
                    &self
                        .db
                        .get_pinned_cf($cf, serialize(&$key).unwrap())?
                        .unwrap(),
                )
                .unwrap()
            }};
        }

        let mut to: H256 = to;
        let mut from: H256 = from;

        let mut to_level: u64 = get_value!(voter_node_level_cf, to);
        let mut from_level: u64 = get_value!(voter_node_level_cf, from);

        let mut added_votes: Vec<(H256, u64)> = vec![];
        let mut removed_votes: Vec<(H256, u64)> = vec![];

        // trace back the longer chain until the levels of the two tips are the same
        while to_level != from_level {
            if to_level > from_level {
                let votes: Vec<H256> = get_value!(vote_neighbor_cf, to);
                for vote in votes {
                    added_votes.push((vote, to_level));
                }
                to = get_value!(voter_parent_neighbor_cf, to);
                to_level -= 1;
            } else if to_level < from_level {
                let votes: Vec<H256> = get_value!(vote_neighbor_cf, from);
                for vote in votes {
                    removed_votes.push((vote, from_level));
                }
                from = get_value!(voter_parent_neighbor_cf, from);
                from_level -= 1;
            }
        }

        while to != from {
            let votes: Vec<H256> = get_value!(vote_neighbor_cf, to);
            for vote in votes {
                added_votes.push((vote, to_level));
            }
            to = get_value!(voter_parent_neighbor_cf, to);
            to_level -= 1;

            let votes: Vec<H256> = get_value!(vote_neighbor_cf, from);
            for vote in votes {
                removed_votes.push((vote, from_level));
            }
            from = get_value!(voter_parent_neighbor_cf, from);
            from_level -= 1;
        }
        return Ok((added_votes, removed_votes));
    }

    pub fn best_proposer(&self) -> Result<H256> {
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();

        let proposer_best = self.proposer_best_level.lock().unwrap();
        let level: u64 = *proposer_best;
        drop(proposer_best);
        let blocks: Vec<H256> = deserialize(
            &self
                .db
                .get_pinned_cf(proposer_tree_level_cf, serialize(&level).unwrap())?
                .unwrap(),
        )
        .unwrap();
        return Ok(blocks[0]);
    }

    pub fn best_voter(&self, chain_num: usize) -> H256 {
        let voter_best = self.voter_best[chain_num].lock().unwrap();
        let hash = voter_best.0;
        drop(voter_best);
        return hash;
    }

    pub fn unreferred_proposers(&self) -> Vec<H256> {
        // TODO: does ordering matter?
        // TODO: should remove the parent block when mining
        let unreferred_proposers = self.unreferred_proposers.lock().unwrap();
        let list: Vec<H256> = unreferred_proposers.iter().cloned().collect();
        drop(unreferred_proposers);
        return list;
    }

    pub fn unreferred_transactions(&self) -> Vec<H256> {
        // TODO: does ordering matter?
        let unreferred_transactions = self.unreferred_transactions.lock().unwrap();
        let list: Vec<H256> = unreferred_transactions.iter().cloned().collect();
        drop(unreferred_transactions);
        return list;
    }

    /// Get the list of unvoted proposer blocks that a voter chain should vote for, given the tip
    /// of the particular voter chain.
    pub fn unvoted_proposer(&self, tip: &H256, proposer_parent: &H256) -> Result<Vec<H256>> {
        let voter_node_voted_level_cf = self.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_node_level_cf = self.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let proposer_vote_count_cf = self.db.cf_handle(PROPOSER_VOTE_COUNT_CF).unwrap();
        // get the deepest voted level
        let first_vote_level: u64 = deserialize(
            &self
                .db
                .get_pinned_cf(voter_node_voted_level_cf, serialize(&tip).unwrap())?
                .unwrap(),
        )
        .unwrap();

        let last_vote_level: u64 = deserialize(
            &self
                .db
                .get_pinned_cf(proposer_node_level_cf, serialize(&proposer_parent).unwrap())?
                .unwrap(),
        )
        .unwrap();

        // get the block with the most votes on each proposer level
        // and break ties with hash value
        let mut list: Vec<H256> = vec![];
        for level in first_vote_level + 1..=last_vote_level {
            let mut blocks: Vec<H256> = deserialize(
                &self
                    .db
                    .get_pinned_cf(proposer_tree_level_cf, serialize(&(level as u64)).unwrap())?
                    .unwrap(),
            )
            .unwrap();
            blocks.sort_unstable();
            // the current best proposer block to vote for
            let mut best_vote: Option<(H256, u64)> = None;
            for block_hash in &blocks {
                let vote_count: u64 = match &self
                    .db
                    .get_pinned_cf(proposer_vote_count_cf, serialize(&block_hash).unwrap())?
                {
                    Some(d) => deserialize(d).unwrap(),
                    None => 0,
                };
                match best_vote {
                    Some((_, num_votes)) => {
                        if vote_count > num_votes {
                            best_vote = Some((*block_hash, vote_count));
                        }
                    }
                    None => {
                        best_vote = Some((*block_hash, vote_count));
                    }
                }
            }
            list.push(best_vote.unwrap().0); //Note: the last vote in list could be other proposer that at the same level of proposer_parent
        }
        return Ok(list);
    }

    /// Get the level of the proposer block
    pub fn proposer_level(&self, hash: &H256) -> Result<u64> {
        let proposer_node_level_cf = self.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let level: u64 = deserialize(
            &self
                .db
                .get_pinned_cf(proposer_node_level_cf, serialize(&hash).unwrap())?
                .unwrap(),
        )
        .unwrap();
        return Ok(level);
    }

    /// Get the deepest voted level of a voter
    pub fn deepest_voted_level(&self, voter: &H256) -> Result<u64> {
        let voter_node_voted_level_cf = self.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        // get the deepest voted level
        let voted_level: u64 = deserialize(
            &self
                .db
                .get_pinned_cf(voter_node_voted_level_cf, serialize(voter).unwrap())?
                .unwrap(),
        )
        .unwrap();
        return Ok(voted_level);
    }

    /// Get the chain number of the voter block
    pub fn voter_chain_number(&self, hash: &H256) -> Result<u16> {
        let voter_node_chain_cf = self.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let chain: u16 = deserialize(
            &self
                .db
                .get_pinned_cf(voter_node_chain_cf, serialize(&hash).unwrap())?
                .unwrap(),
        )
        .unwrap();
        return Ok(chain);
    }

    /// Check whether the given proposer block exists in the database.
    pub fn contains_proposer(&self, hash: &H256) -> Result<bool> {
        let proposer_node_level_cf = self.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        return match self
            .db
            .get_pinned_cf(proposer_node_level_cf, serialize(&hash).unwrap())?
        {
            Some(_) => Ok(true),
            None => Ok(false),
        };
    }

    /// Check whether the given voter block exists in the database.
    pub fn contains_voter(&self, hash: &H256) -> Result<bool> {
        let voter_node_level_cf = self.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        return match self
            .db
            .get_pinned_cf(voter_node_level_cf, serialize(&hash).unwrap())?
        {
            Some(_) => Ok(true),
            None => Ok(false),
        };
    }

    /// Check whether the given transaction block exists in the database.
    // TODO: we can't tell whether it's is a transaction block!
    pub fn contains_transaction(&self, hash: &H256) -> Result<bool> {
        let parent_neighbor_cf = self.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        return match self
            .db
            .get_pinned_cf(parent_neighbor_cf, serialize(&hash).unwrap())?
        {
            Some(_) => Ok(true),
            None => Ok(false),
        };
    }

    pub fn proposer_leaders(&self) -> Result<Vec<H256>> {
        let proposer_leader_sequence_cf = self.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_ledger_tip = self.proposer_ledger_tip.lock().unwrap();
        let snapshot = self.db.snapshot();
        let ledger_tip_level = *proposer_ledger_tip;
        let mut leaders = vec![];
        drop(proposer_ledger_tip);
        for level in 0..=ledger_tip_level {
            match snapshot.get_cf(proposer_leader_sequence_cf, serialize(&level).unwrap())? {
                Some(d) => {
                    let hash: H256 = deserialize(&d).unwrap();
                    leaders.push(hash);
                }
                None => unreachable!(),
            }
        }
        return Ok(leaders);
    }
}

impl BlockChain {
    pub fn proposer_transaction_in_ledger(&self, limit: u64) -> Result<Vec<(H256, Vec<H256>)>> {
        let ledger_tip_ = self.proposer_ledger_tip.lock().unwrap();
        let ledger_tip = *ledger_tip_;
        // TODO: get snapshot here doesn't ensure consistency of snapshot, since we use multiple write batch in `insert_block`
        // and the ledger_tip lock doesn't ensure it either.
        let snapshot = self.db.snapshot();
        drop(ledger_tip_);

        let ledger_bottom: u64 = match ledger_tip > limit {
            true => ledger_tip - limit,
            false => 0,
        };
        let proposer_ledger_order_cf = self.db.cf_handle(PROPOSER_LEDGER_ORDER_CF).unwrap();
        let transaction_ref_neighbor_cf = self.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();

        let mut proposer_in_ledger: Vec<H256> = vec![];
        let mut ledger: Vec<(H256, Vec<H256>)> = vec![];

        for level in ledger_bottom..=ledger_tip {
            match snapshot.get_cf(proposer_ledger_order_cf, serialize(&level).unwrap())? {
                Some(d) => {
                    let mut blocks: Vec<H256> = deserialize(&d).unwrap();
                    proposer_in_ledger.append(&mut blocks);
                }
                None => {
                    unreachable!("level <= ledger tip should exist in proposer_ledger_order_cf")
                }
            }
        }

        for hash in &proposer_in_ledger {
            let blocks: Vec<H256> = match snapshot.get_cf(transaction_ref_neighbor_cf, serialize(&hash).unwrap())? {
                Some(d) => deserialize(&d).unwrap(),
                None => unreachable!("proposer in ledger should have transaction ref in database (even for empty ref)"),
            };
            ledger.push((*hash, blocks));
        }
        Ok(ledger)
    }

    pub fn proposer_bottom_tip(&self) -> Result<(H256, H256, u64)> {
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let proposer_bottom = match self
            .db
            .get_pinned_cf(proposer_tree_level_cf, serialize(&1u64).unwrap())?
        {
            Some(d) => {
                let blocks: Vec<H256> = deserialize(&d).unwrap();
                blocks.into_iter().next()
            }
            None => None,
        };
        if let Some(proposer_bottom) = proposer_bottom {
            let proposer_best = self.proposer_best_level.lock().unwrap();
            let proposer_best_level = *proposer_best;
            drop(proposer_best);
            let proposer_tip = match self.db.get_pinned_cf(
                proposer_tree_level_cf,
                serialize(&proposer_best_level).unwrap(),
            )? {
                Some(d) => {
                    let blocks: Vec<H256> = deserialize(&d).unwrap();
                    blocks[0]
                }
                None => unreachable!(),
            };
            return Ok((proposer_bottom, proposer_tip, proposer_best_level));
        } else {
            return Ok((H256::default(), H256::default(), 0));
        }
    }

    pub fn voter_bottom_tip(&self) -> Result<Vec<(H256, H256, u64)>> {
        let voter_node_chain_cf = self.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_level_cf = self.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let iter = self
            .db
            .iterator_cf(voter_node_chain_cf, rocksdb::IteratorMode::Start)?;
        // vector of pair (level-1 voter, best voter, best level)
        let mut voters = vec![(H256::default(), H256::default(), 0u64); self.voter_best.len()];
        for (k, v) in iter {
            let hash: H256 = deserialize(k.as_ref()).unwrap();
            let chain: u16 = deserialize(v.as_ref()).unwrap();
            let level: u64 = match self.db.get_pinned_cf(voter_node_level_cf, k.as_ref())? {
                Some(d) => deserialize(&d).unwrap(),
                None => unreachable!("voter should have level"),
            };
            if level == 1 {
                voters[chain as usize].0 = hash;
            }
        }
        for chain in 0..self.voter_best.len() {
            let voter_best = self.voter_best[chain].lock().unwrap();
            voters[chain as usize].1 = voter_best.0;
            voters[chain as usize].2 = voter_best.1;
        }
        Ok(voters)
    }

    pub fn dump(&self, limit: u64, display_fork: bool) -> Result<String> {
        /// Struct to hold blockchain data to be dumped
        #[derive(Serialize)]
        struct Dump {
            edges: Vec<Edge>,
            proposer_levels: Vec<Vec<String>>,
            proposer_leaders: BTreeMap<u64, String>,
            voter_longest: Vec<String>,
            proposer_nodes: HashMap<String, Proposer>,
            voter_nodes: HashMap<String, Voter>,
            //pub transaction_unconfirmed: Vec<String>, //what is the definition of this?
            transaction_in_ledger: Vec<String>,
            transaction_unreferred: Vec<String>,
            proposer_in_ledger: Vec<String>,
            voter_chain_number: Vec<String>,
            proposer_tree_number: String,
        }

        #[derive(Serialize)]
        enum EdgeType {
            VoterToVoterParent,
            ProposerToProposerParent,
            VoterToProposerVote,
        }
        #[derive(Serialize)]
        struct Edge {
            from: String,
            to: String,
            edgetype: EdgeType,
        }

        #[derive(Serialize)]
        struct Proposer {
            level: u64,
            status: ProposerStatus,
            votes: u16,
        }

        #[derive(Serialize)]
        enum ProposerStatus {
            Leader,
            Others, //don't know what other status we need for proposer? like unconfirmed? unreferred?
        }

        #[derive(Serialize)]
        struct Voter {
            chain: u16,
            level: u64,
            status: VoterStatus,
            deepest_vote_level: u64,
        }

        #[derive(Serialize)]
        enum VoterStatus {
            OnMainChain,
            Orphan,
        }

        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let proposer_leader_sequence_cf = self.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let parent_neighbor_cf = self.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let proposer_node_vote_cf = self.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();
        let voter_parent_neighbor_cf = self.db.cf_handle(VOTER_PARENT_NEIGHBOR_CF).unwrap();
        let voter_node_voted_level_cf = self.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_ledger_order_cf = self.db.cf_handle(PROPOSER_LEDGER_ORDER_CF).unwrap();
        let transaction_ref_neighbor_cf = self.db.cf_handle(TRANSACTION_REF_NEIGHBOR_CF).unwrap();
        let voter_node_chain_cf = self.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_level_cf = self.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_neighbor_cf = self.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();

        // for computing the lowest level for voter chains related to the 100 levels of proposer nodes
        let mut voter_lowest: Vec<u64> = vec![];
        // get voter best blocks and levels, this is from memory
        let mut voter_longest: Vec<(H256, u64)> = vec![];
        // total voter number for each chain (using db iteration)
        let mut voter_number: Vec<u64> = vec![];

        // get the ledger tip and bottom, for processing ledger
        let ledger_tip_ = self.proposer_ledger_tip.lock().unwrap();
        let ledger_tip = *ledger_tip_;
        for voter_chain in self.voter_best.iter() {
            let longest = voter_chain.lock().unwrap();
            voter_longest.push((longest.0, longest.1));
            voter_lowest.push(longest.1);
            voter_number.push(0);
        }
        // TODO: get snapshot here doesn't ensure consistency of snapshot, since we use multiple write batch in `insert_block`
        // and the ledger_tip lock doesn't ensure it either.
        let snapshot = self.db.snapshot();
        drop(ledger_tip_);
        let ledger_bottom: u64 = match ledger_tip > limit {
            true => ledger_tip - limit,
            false => 0,
        };

        // memory cache for votes
        let mut vote_cache: HashMap<(u16, u64), Vec<H256>> = HashMap::new();

        let mut edges: Vec<Edge> = vec![];
        let mut proposer_tree: BTreeMap<u64, Vec<H256>> = BTreeMap::new();
        let mut proposer_nodes: HashMap<String, Proposer> = HashMap::new();
        let mut voter_nodes: HashMap<String, Voter> = HashMap::new();
        let mut proposer_leaders: BTreeMap<u64, String> = BTreeMap::new();
        let mut proposer_in_ledger: Vec<H256> = vec![];
        let mut transaction_in_ledger: Vec<String> = vec![];

        // proposer tree
        for level in ledger_bottom.. {
            match snapshot.get_cf(proposer_tree_level_cf, serialize(&level).unwrap())? {
                Some(d) => {
                    let blocks: Vec<H256> = deserialize(&d).unwrap();
                    proposer_tree.insert(level, blocks);
                }
                None => break,
            }
        }

        // one pass of proposer. get proposer node info, cache votes.
        for (level, blocks) in proposer_tree.iter() {
            for block in blocks {
                // get parent edges
                match snapshot.get_cf(parent_neighbor_cf, serialize(block).unwrap())? {
                    Some(d) => {
                        let parent: H256 = deserialize(&d).unwrap();
                        edges.push(Edge {
                            from: block.to_string(),
                            to: parent.to_string(),
                            edgetype: EdgeType::ProposerToProposerParent,
                        });
                    }
                    None => {}
                }
                // get proposer node info
                match snapshot.get_cf(proposer_node_vote_cf, serialize(block).unwrap())? {
                    Some(d) => {
                        let votes: Vec<(u16, u64)> = deserialize(&d).unwrap();
                        proposer_nodes.insert(
                            block.to_string(),
                            Proposer {
                                level: *level,
                                status: ProposerStatus::Others,
                                votes: votes.len() as u16,
                            },
                        );

                        // get voter edges
                        for (chain, level) in &votes {
                            let lowest = voter_lowest
                                .get_mut(*chain as usize)
                                .expect("should've computed lowest level");
                            if *lowest > *level {
                                *lowest = *level;
                            }

                            // cache the votes
                            if let Some(v) = vote_cache.get_mut(&(*chain, *level)) {
                                v.push(*block);
                            } else {
                                vote_cache.insert((*chain, *level), vec![*block]);
                            }
                        }
                    }
                    None => {
                        proposer_nodes.insert(
                            block.to_string(),
                            Proposer {
                                level: *level,
                                status: ProposerStatus::Others,
                                votes: 0, //no votes for proposer in database, so 0 vote
                            },
                        );
                    }
                }
            }
        }

        // one pass of voters. get voter info, voter parent, and vote edges. notice the votes are cached
        for (chain, longest) in voter_longest.iter().enumerate() {
            let mut voter_block = longest.0;
            let mut level = longest.1;
            let lowest = voter_lowest
                .get(chain)
                .expect("should've computed lowest level");
            while level >= *lowest {
                // voter info
                let deepest_vote_level: u64 = match snapshot
                    .get_cf(voter_node_voted_level_cf, serialize(&voter_block).unwrap())?
                {
                    Some(d) => deserialize(&d).unwrap(),
                    None => unreachable!("voter block should have voted level in database"),
                };
                voter_nodes.insert(
                    voter_block.to_string(),
                    Voter {
                        chain: chain as u16,
                        level,
                        status: VoterStatus::OnMainChain,
                        deepest_vote_level,
                    },
                );
                // vote edges
                if let Some(votes) = vote_cache.get(&(chain as u16, level)) {
                    for vote in votes {
                        edges.push(Edge {
                            from: voter_block.to_string(),
                            to: vote.to_string(),
                            edgetype: EdgeType::VoterToProposerVote,
                        });
                    }
                }
                // voter parent
                match snapshot.get_cf(voter_parent_neighbor_cf, serialize(&voter_block).unwrap())? {
                    Some(d) => {
                        let parent: H256 = deserialize(&d).unwrap();
                        edges.push(Edge {
                            from: voter_block.to_string(),
                            to: parent.to_string(),
                            edgetype: EdgeType::VoterToVoterParent,
                        });
                        voter_block = parent;
                        level -= 1;
                    }
                    None => {
                        break; // if no parent, must break the while loop
                    }
                }
            }
        }

        // use iterator to find voter fork and orphan voters, may be slow
        if display_fork {
            let iter = snapshot.iterator_cf(voter_node_chain_cf, rocksdb::IteratorMode::Start)?;
            for (k, v) in iter {
                let hash: H256 = deserialize(k.as_ref()).unwrap();
                let voter_block = hash.to_string();
                let chain: u16 = deserialize(v.as_ref()).unwrap();
                voter_number[chain as usize] += 1;
                let level: u64 = match snapshot.get_cf(voter_node_level_cf, k.as_ref())? {
                    Some(d) => deserialize(&d).unwrap(),
                    None => unreachable!("voter should have level"),
                };
                let lowest = voter_lowest
                    .get(chain as usize)
                    .expect("should've computed lowest level");
                if level >= *lowest && !voter_nodes.contains_key(&voter_block) {
                    let deepest_vote_level: u64 =
                        match snapshot.get_cf(voter_node_voted_level_cf, k.as_ref())? {
                            Some(d) => deserialize(&d).unwrap(),
                            None => unreachable!("voter block should have voted level in database"),
                        };
                    voter_nodes.insert(
                        voter_block.clone(),
                        Voter {
                            chain,
                            level,
                            status: VoterStatus::Orphan,
                            deepest_vote_level,
                        },
                    );
                    // vote edges
                    match snapshot.get_cf(voter_neighbor_cf, k.as_ref())? {
                        Some(d) => {
                            let votes: Vec<H256> = deserialize(&d).unwrap();
                            for vote in &votes {
                                edges.push(Edge {
                                    from: voter_block.clone(),
                                    to: vote.to_string(),
                                    edgetype: EdgeType::VoterToProposerVote,
                                });
                            }
                        }
                        None => unreachable!("voter block should have votes level in database"),
                    }
                    // voter parent
                    match snapshot.get_cf(voter_parent_neighbor_cf, k.as_ref())? {
                        Some(d) => {
                            let parent: H256 = deserialize(&d).unwrap();
                            edges.push(Edge {
                                from: voter_block.clone(),
                                to: parent.to_string(),
                                edgetype: EdgeType::VoterToVoterParent,
                            });
                        }
                        None => {}
                    }
                }
            }
        }

        // proposer leader
        for level in proposer_tree.keys() {
            match snapshot.get_cf(proposer_leader_sequence_cf, serialize(level).unwrap())? {
                Some(d) => {
                    let h256: H256 = deserialize(&d).unwrap();
                    proposer_leaders.insert(*level, h256.to_string());
                    if let Some(proposer) = proposer_nodes.get_mut(&h256.to_string()) {
                        proposer.status = ProposerStatus::Leader;
                    }
                }
                None => {}
            }
        }

        // ledger
        for level in ledger_bottom..=ledger_tip {
            match snapshot.get_cf(
                proposer_ledger_order_cf,
                serialize(&(level as u64)).unwrap(),
            )? {
                Some(d) => {
                    let mut blocks: Vec<H256> = deserialize(&d).unwrap();
                    proposer_in_ledger.append(&mut blocks);
                }
                None => {
                    unreachable!("level <= ledger tip should exist in proposer_ledger_order_cf")
                }
            }
        }

        for hash in &proposer_in_ledger {
            match snapshot.get_cf(transaction_ref_neighbor_cf, serialize(&hash).unwrap())? {
                Some(d) => {
                    let blocks: Vec<H256> = deserialize(&d).unwrap();
                    let mut blocks = blocks.into_iter().map(|h|h.to_string()).collect();
                    transaction_in_ledger.append(&mut blocks);
                }
                None => unreachable!("proposer in ledger should have transaction ref in database (even for empty ref)"),
            }
        }

        // TODO: transaction_unreferred may be inconsistent with other things
        let transaction_unreferred_ = self.unreferred_transactions.lock().unwrap();
        let transaction_unreferred: Vec<String> = transaction_unreferred_
            .iter()
            .map(|h| h.to_string())
            .collect();
        drop(transaction_unreferred_);

        // proposer number
        let mut proposer_number: usize = 0;
        let mut proposer_level: u64 = 0;
        for level in 0u64.. {
            match snapshot.get_cf(proposer_tree_level_cf, serialize(&level).unwrap())? {
                Some(d) => {
                    let blocks: Vec<H256> = deserialize(&d).unwrap();
                    proposer_number += blocks.len();
                }
                None => {
                    proposer_level = level;
                    break;
                }
            }
        }
        let proposer_tree_number = format!("({}/{}) ", proposer_level, proposer_number);

        // voter numbers
        let mut voter_chain_number: Vec<String> = vec![];
        for (chain, longest) in voter_longest.iter().enumerate() {
            if voter_number[chain] == 0 {
                voter_chain_number.push("".to_string());
            } else {
                voter_chain_number.push(format!("({}/{}) ", 1 + longest.1, voter_number[chain]));
            }
        }

        let proposer_levels: Vec<Vec<String>> = proposer_tree
            .into_iter()
            .map(|(_k, v)| v.into_iter().map(|h256| h256.to_string()).collect())
            .collect();
        let voter_longest: Vec<String> = voter_longest
            .into_iter()
            .map(|(h, _u)| h.to_string())
            .collect();
        let proposer_in_ledger: Vec<String> = proposer_in_ledger
            .into_iter()
            .map(|h| h.to_string())
            .collect();
        // filter the edges for nodes_to_show
        let mut proposer_to_show: Vec<String> = proposer_nodes.keys().cloned().collect();
        let mut voter_to_show: Vec<String> = voter_nodes.keys().cloned().collect();
        proposer_to_show.append(&mut voter_to_show);
        let nodes_to_show: HashSet<String> = proposer_to_show.into_iter().collect();
        let edges: Vec<Edge> = edges
            .into_iter()
            .filter(|e| nodes_to_show.contains(&e.from) && nodes_to_show.contains(&e.to))
            .collect();

        let dump = Dump {
            edges,
            proposer_levels,
            proposer_leaders,
            voter_longest,
            proposer_nodes,
            voter_nodes,
            proposer_in_ledger,
            transaction_in_ledger,
            transaction_unreferred,
            voter_chain_number,
            proposer_tree_number,
        };

        Ok(serde_json::to_string_pretty(&dump).unwrap())
    }
}

fn vote_vec_merge(
    _: &[u8],
    existing_val: Option<&[u8]>,
    operands: &mut rocksdb::merge_operator::MergeOperands,
) -> Option<Vec<u8>> {
    let mut existing: Vec<(u16, u64)> = match existing_val {
        Some(v) => deserialize(v).unwrap(),
        None => vec![],
    };
    for op in operands {
        // parse the operation as add(true)/remove(false), chain(u16), level(u64)
        let operation: (bool, u16, u64) = deserialize(op).unwrap();
        match operation.0 {
            true => {
                if !existing.contains(&(operation.1, operation.2)) {
                    existing.push((operation.1, operation.2));
                }
            }
            false => {
                match existing.iter().position(|&x| x.0 == operation.1) {
                    Some(p) => existing.swap_remove(p),
                    None => continue, // TODO: potential bug here - what if we delete a nonexisting item
                };
            }
        }
    }
    let result: Vec<u8> = serialize(&existing).unwrap();
    return Some(result);
}

fn h256_vec_append_merge(
    _: &[u8],
    existing_val: Option<&[u8]>,
    operands: &mut rocksdb::merge_operator::MergeOperands,
) -> Option<Vec<u8>> {
    let mut existing: Vec<H256> = match existing_val {
        Some(v) => deserialize(v).unwrap(),
        None => vec![],
    };
    for op in operands {
        let new_hash: H256 = deserialize(op).unwrap();
        if !existing.contains(&new_hash) {
            existing.push(new_hash);
        }
    }
    let result: Vec<u8> = serialize(&existing).unwrap();
    return Some(result);
}

fn u64_plus_merge(
    _: &[u8],
    existing_val: Option<&[u8]>,
    operands: &mut rocksdb::merge_operator::MergeOperands,
) -> Option<Vec<u8>> {
    let mut existing: u64 = match existing_val {
        Some(v) => deserialize(v).unwrap(),
        None => 0,
    };
    for op in operands {
        let to_add: u64 = deserialize(op).unwrap();
        existing = existing + to_add;
    }
    let result: Vec<u8> = serialize(&existing).unwrap();
    return Some(result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{proposer, transaction, voter, Block, Content};
    use crate::crypto::hash::H256;

    #[test]
    fn initialize_new() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_new.rocksdb").unwrap();
        // get cf handles
        let proposer_node_vote_cf = db.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();
        let proposer_node_level_cf = db.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let voter_node_level_cf = db.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_chain_cf = db.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = db.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = db.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = db.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_ledger_order_cf = db.db.cf_handle(PROPOSER_LEDGER_ORDER_CF).unwrap();

        // validate proposer genesis
        let genesis_level: u64 = deserialize(
            &db.db
                .get_pinned_cf(
                    proposer_node_level_cf,
                    serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
                )
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(genesis_level, 0);
        let level_0_blocks: Vec<H256> = deserialize(
            &db.db
                .get_pinned_cf(proposer_tree_level_cf, serialize(&(0 as u64)).unwrap())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(level_0_blocks, vec![*PROPOSER_GENESIS_HASH]);
        let genesis_votes: Vec<(u16, u64)> = deserialize(
            &db.db
                .get_pinned_cf(
                    proposer_node_vote_cf,
                    serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(),
                )
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        let mut true_genesis_votes: Vec<(u16, u64)> = vec![];
        for chain_num in 0..NUM_VOTER_CHAINS {
            true_genesis_votes.push((chain_num as u16, 0));
        }
        assert_eq!(genesis_votes, true_genesis_votes);
        assert_eq!(*db.proposer_best_level.lock().unwrap(), 0);
        assert_eq!(*db.unconfirmed_proposers.lock().unwrap(), HashSet::new());
        assert_eq!(db.unreferred_proposers.lock().unwrap().len(), 1);
        assert_eq!(
            db.unreferred_proposers
                .lock()
                .unwrap()
                .contains(&(PROPOSER_GENESIS_HASH)),
            true
        );
        let level_0_leader: H256 = deserialize(
            &db.db
                .get_pinned_cf(proposer_leader_sequence_cf, serialize(&(0 as u64)).unwrap())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(level_0_leader, *PROPOSER_GENESIS_HASH);
        let level_0_confirms: Vec<H256> = deserialize(
            &db.db
                .get_pinned_cf(proposer_ledger_order_cf, serialize(&(0 as u64)).unwrap())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(level_0_confirms, vec![*PROPOSER_GENESIS_HASH]);

        // validate voter genesis
        for chain_num in 0..NUM_VOTER_CHAINS {
            let genesis_level: u64 = deserialize(
                &db.db
                    .get_pinned_cf(
                        voter_node_level_cf,
                        serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                    )
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(genesis_level, 0);
            let voted_level: u64 = deserialize(
                &db.db
                    .get_pinned_cf(
                        voter_node_voted_level_cf,
                        serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                    )
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(voted_level, 0);
            let genesis_chain: u16 = deserialize(
                &db.db
                    .get_pinned_cf(
                        voter_node_chain_cf,
                        serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                    )
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(genesis_chain, chain_num as u16);
            let parent: H256 = deserialize(
                &db.db
                    .get_pinned_cf(
                        parent_neighbor_cf,
                        serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                    )
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(parent, *PROPOSER_GENESIS_HASH);
            let voted_proposer: Vec<H256> = deserialize(
                &db.db
                    .get_pinned_cf(
                        vote_neighbor_cf,
                        serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(),
                    )
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(voted_proposer, vec![*PROPOSER_GENESIS_HASH]);
            assert_eq!(
                *db.voter_best[chain_num as usize].lock().unwrap(),
                (VOTER_GENESIS_HASHES[chain_num as usize], 0)
            );
        }
    }

    #[test]
    fn best_proposer_and_voter() {
        let db =
            BlockChain::new("/tmp/prism_test_blockchain_best_proposer_and_voter.rocksdb").unwrap();
        assert_eq!(db.best_proposer().unwrap(), *PROPOSER_GENESIS_HASH);
        assert_eq!(db.best_voter(0), VOTER_GENESIS_HASHES[0]);

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [0; 32],
            H256::default(),
        );
        db.insert_block(&new_proposer_block).unwrap();
        let new_voter_content = Content::Voter(voter::Content::new(
            0,
            VOTER_GENESIS_HASHES[0],
            vec![new_proposer_block.hash()],
        ));
        let new_voter_block = Block::new(
            new_proposer_block.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [1; 32],
            H256::default(),
        );
        db.insert_block(&new_voter_block).unwrap();
        assert_eq!(db.best_proposer().unwrap(), new_proposer_block.hash());
        assert_eq!(db.best_voter(0), new_voter_block.hash());
    }

    #[test]
    fn unreferred_transactions_and_proposer() {
        let db =
            BlockChain::new("/tmp/prism_test_blockchain_unreferred_transactions_proposer.rocksdb")
                .unwrap();

        let new_transaction_content = Content::Transaction(transaction::Content::new(vec![]));
        let new_transaction_block = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_transaction_content,
            [0; 32],
            H256::default(),
        );
        db.insert_block(&new_transaction_block).unwrap();
        assert_eq!(
            db.unreferred_transactions(),
            vec![new_transaction_block.hash()]
        );
        assert_eq!(db.unreferred_proposers(), vec![*PROPOSER_GENESIS_HASH]);

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block_1 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [1; 32],
            H256::default(),
        );
        db.insert_block(&new_proposer_block_1).unwrap();
        assert_eq!(
            db.unreferred_transactions(),
            vec![new_transaction_block.hash()]
        );
        assert_eq!(db.unreferred_proposers(), vec![new_proposer_block_1.hash()]);

        let new_proposer_content = Content::Proposer(proposer::Content::new(
            vec![new_transaction_block.hash()],
            vec![new_proposer_block_1.hash()],
        ));
        let new_proposer_block_2 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [2; 32],
            H256::default(),
        );
        db.insert_block(&new_proposer_block_2).unwrap();
        assert_eq!(db.unreferred_transactions(), vec![]);
        assert_eq!(db.unreferred_proposers(), vec![new_proposer_block_2.hash()]);
    }

    #[test]
    fn unvoted_proposer() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_unvoted_proposer.rocksdb").unwrap();
        assert_eq!(
            db.unvoted_proposer(&VOTER_GENESIS_HASHES[0], &db.best_proposer().unwrap())
                .unwrap(),
            vec![]
        );

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block_1 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [0; 32],
            H256::default(),
        );
        db.insert_block(&new_proposer_block_1).unwrap();

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block_2 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [1; 32],
            H256::default(),
        );
        db.insert_block(&new_proposer_block_2).unwrap();
        assert_eq!(
            db.unvoted_proposer(&VOTER_GENESIS_HASHES[0], &db.best_proposer().unwrap())
                .unwrap(),
            vec![new_proposer_block_1.hash()]
        );

        let new_voter_content = Content::Voter(voter::Content::new(
            0,
            VOTER_GENESIS_HASHES[0],
            vec![new_proposer_block_1.hash()],
        ));
        let new_voter_block = Block::new(
            new_proposer_block_2.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [2; 32],
            H256::default(),
        );
        db.insert_block(&new_voter_block).unwrap();

        assert_eq!(
            db.unvoted_proposer(&VOTER_GENESIS_HASHES[0], &db.best_proposer().unwrap())
                .unwrap(),
            vec![new_proposer_block_1.hash()]
        );
        assert_eq!(
            db.unvoted_proposer(&new_voter_block.hash(), &db.best_proposer().unwrap())
                .unwrap(),
            vec![]
        );
    }

    #[test]
    fn merge_operator_h256_vec() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_merge_op_h256_vec.rocksdb").unwrap();
        let cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();

        let hash_1: H256 = [0u8; 32].into();
        let hash_2: H256 = [1u8; 32].into();
        let hash_3: H256 = [2u8; 32].into();
        // merge with an nonexistent entry
        db.db
            .merge_cf(cf, b"testkey", serialize(&hash_1).unwrap())
            .unwrap();
        let result: Vec<H256> =
            deserialize(&db.db.get_pinned_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![hash_1]);

        // merge with an existing entry
        db.db
            .merge_cf(cf, b"testkey", serialize(&hash_2).unwrap())
            .unwrap();
        db.db
            .merge_cf(cf, b"testkey", serialize(&hash_3).unwrap())
            .unwrap();
        let result: Vec<H256> =
            deserialize(&db.db.get_pinned_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![hash_1, hash_2, hash_3]);
    }

    #[test]
    fn merge_operator_btreemap() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_merge_op_u64_vec.rocksdb").unwrap();
        let cf = db.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();

        // merge with an nonexistent entry
        db.db
            .merge_cf(
                cf,
                b"testkey",
                serialize(&(true, 0 as u16, 0 as u64)).unwrap(),
            )
            .unwrap();
        let result: Vec<(u16, u64)> =
            deserialize(&db.db.get_pinned_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![(0, 0)]);

        // insert
        db.db
            .merge_cf(
                cf,
                b"testkey",
                serialize(&(true, 10 as u16, 0 as u64)).unwrap(),
            )
            .unwrap();
        db.db
            .merge_cf(
                cf,
                b"testkey",
                serialize(&(true, 5 as u16, 0 as u64)).unwrap(),
            )
            .unwrap();
        let result: Vec<(u16, u64)> =
            deserialize(&db.db.get_pinned_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![(0, 0), (10, 0), (5, 0)]);

        // remove
        db.db
            .merge_cf(
                cf,
                b"testkey",
                serialize(&(false, 5 as u16, 0 as u64)).unwrap(),
            )
            .unwrap();
        let result: Vec<(u16, u64)> =
            deserialize(&db.db.get_pinned_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![(0, 0), (10, 0)]);
    }
}
