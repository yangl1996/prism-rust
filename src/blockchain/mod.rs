use crate::crypto::hash::{Hashable, H256};
use crate::block::{Block, Content};
use crate::config::*;
 
use std::sync::Mutex;
use bincode::{deserialize, serialize};
use rocksdb::{Options, ColumnFamilyDescriptor, DB, WriteBatch};
use std::collections::HashSet;

// Column family names for node/chain metadata
const PROPOSER_NODE_LEVEL_CF: &str = "PROPOSER_NODE_LEVEL";             // hash to node level (u64)
const VOTER_NODE_LEVEL_CF: &str = "VOTER_NODE_LEVEL";                   // hash to node level (u64)
const VOTER_NODE_CHAIN_CF: &str = "VOTER_NODE_CHAIN";                   // hash to chain number (u16)
const PROPOSER_TREE_LEVEL_CF: &str = "PROPOSER_TREE_LEVEL";             // level (u64) to hashes of blocks (Vec<hash>)
const VOTER_NODE_VOTED_LEVEL_CF: &str = "VOTER_NODE_VOTED_LEVEL";       // hash to max. voted level (u64)
const PROPOSER_NODE_VOTE_CF: &str = "PROPOSER_NODE_VOTE";               // hash to level and chain number of main chain votes (Vec<u16, u64>)
const PROPOSER_LEADER_SEQUENCE_CF: &str = "PROPOSER_LEADER_SEQUENCE";   // level (u64) to hash of leader block.
const PROPOSER_CONFIRM_LIST_CF: &str = "PROPOSER_CONFIRM_LIST";         // level (u64) to the list of proposer blocks confirmed
                                                                        // by this level. The list is in the order that those
                                                                        // blocks should live in the ledger.

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
    unreferred_transaction: Mutex<HashSet<H256>>,
    unreferred_proposer: Mutex<HashSet<H256>>,
}

// Functions to edit the blockchain
impl BlockChain {
    /// Open the blockchain database at the given path, and create missing column families.
    /// This function also populates the metadata fields with default values, and those
    /// fields must be initialized later.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let proposer_node_level_cf = ColumnFamilyDescriptor::new(PROPOSER_NODE_LEVEL_CF, Options::default());
        let voter_node_level_cf = ColumnFamilyDescriptor::new(VOTER_NODE_LEVEL_CF, Options::default());
        let voter_node_chain_cf = ColumnFamilyDescriptor::new(VOTER_NODE_CHAIN_CF, Options::default());
        let voter_node_voted_level_cf = ColumnFamilyDescriptor::new(VOTER_NODE_VOTED_LEVEL_CF, Options::default());
        let proposer_leader_sequence_cf = ColumnFamilyDescriptor::new(PROPOSER_LEADER_SEQUENCE_CF, Options::default());
        let proposer_confirm_list_cf = ColumnFamilyDescriptor::new(PROPOSER_CONFIRM_LIST_CF, Options::default());

        let mut proposer_tree_level_option = Options::default();
        proposer_tree_level_option.set_merge_operator("append H256 vec", h256_vec_append_merge, None);
        let proposer_tree_level_cf = ColumnFamilyDescriptor::new(PROPOSER_TREE_LEVEL_CF, proposer_tree_level_option);

        let mut proposer_node_vote_option = Options::default();
        proposer_node_vote_option.set_merge_operator("insert or remove vote", vote_vec_merge, None);
        let proposer_node_vote_cf = ColumnFamilyDescriptor::new(PROPOSER_NODE_VOTE_CF, proposer_node_vote_option);

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
            voter_node_level_cf,
            voter_node_chain_cf,
            voter_node_voted_level_cf,
            proposer_leader_sequence_cf,
            proposer_confirm_list_cf,
            proposer_tree_level_cf,
            proposer_node_vote_cf,
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
            unreferred_transaction: Mutex::new(HashSet::new()),
            unreferred_proposer: Mutex::new(HashSet::new()),
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
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_confirm_list_cf = db.db.cf_handle(PROPOSER_CONFIRM_LIST_CF).unwrap();

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
        let mut unreferred_proposer = db.unreferred_proposer.lock().unwrap();
        unreferred_proposer.insert(*PROPOSER_GENESIS_HASH);
        drop(unreferred_proposer);
        wb.put_cf(proposer_leader_sequence_cf, serialize(&(0 as u64)).unwrap(),
                  serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;
        let proposer_genesis_confirms: Vec<H256> = vec![];
        wb.put_cf(proposer_confirm_list_cf, serialize(&(0 as u64)).unwrap(),
                  serialize(&proposer_genesis_confirms).unwrap())?;

        // voter genesis blocks
        for chain_num in 0..NUM_VOTER_CHAINS {
            wb.put_cf(parent_neighbor_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(), 
                      serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;
            wb.merge_cf(vote_neighbor_cf, serialize(&VOTER_GENESIS_HASHES[chain_num as usize]).unwrap(), 
                      serialize(&(*PROPOSER_GENESIS_HASH)).unwrap())?;
            wb.merge_cf(proposer_node_vote_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap(), 
                      serialize(&(true, chain_num as u16, 0 as u64)).unwrap())?;
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
        let voter_node_level_cf = self.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_chain_cf = self.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = self.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let proposer_node_vote_cf = self.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();
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
                // remove ref'ed blocks from unreferred list, mark itself as unreferred
                let mut unreferred_proposer = self.unreferred_proposer.lock().unwrap();
                for ref_hash in &content.proposer_refs {
                    unreferred_proposer.remove(&ref_hash);
                }
                unreferred_proposer.remove(&parent_hash);
                unreferred_proposer.insert(block_hash);
                drop(unreferred_proposer);
                let mut unreferred_transaction = self.unreferred_transaction.lock().unwrap();
                for ref_hash in &content.transaction_refs {
                    unreferred_transaction.remove(&ref_hash);
                }
                drop(unreferred_transaction);
                // add ref'ed blocks
                // note that the parent is the first proposer block that we refer
                let mut refed_proposer: Vec<H256> = vec![parent_hash];
                refed_proposer.extend(&content.proposer_refs);
                wb.put_cf(proposer_ref_neighbor_cf, serialize(&block_hash).unwrap(),
                          serialize(&refed_proposer).unwrap())?;
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
                    let voted_level: u64 = deserialize(&self.db.get_cf(proposer_node_level_cf,
                                                                       serialize(&vote_hash).unwrap())?
                                                       .unwrap()).unwrap();
                    if voted_level > deepest_voted_level {
                        deepest_voted_level = voted_level;
                    }
                }
                wb.put_cf(voter_node_voted_level_cf, serialize(&block_hash).unwrap(), 
                          serialize(&deepest_voted_level).unwrap())?;

                let mut voter_best = self.voter_best[self_chain as usize].lock().unwrap();
                let previous_best = voter_best.0;
                let previous_best_level = voter_best.1;
                // update best block
                if self_level > voter_best.1 {
                    voter_best.0 = block_hash;
                    voter_best.1 = self_level;
                }
                drop(voter_best);

                // update vote levels
                if voter_parent_hash == previous_best {
                    // if we just attached to the main chain
                    for added_vote in &content.votes {
                        wb.merge_cf(proposer_node_vote_cf, serialize(&added_vote).unwrap(), serialize(&(true, self_chain as u16, self_level as u64)).unwrap())?;
                    }
                } else {
                    if self_level > previous_best_level {
                        // if it's a side chain, and we are now better than the previous best, then
                        // we are now the new main chain.
                        let mut to_level = voter_parent_level;
                        let mut from_level = previous_best_level;
                        let mut to = voter_parent_hash;
                        let mut from = previous_best;
                        let mut added: Vec<(H256, u64)> = vec![];
                        let mut removed: Vec<(H256, u64)> = vec![];
                        while to_level > from_level {
                            let votes: Vec<H256> = deserialize(&self.db.get_cf(vote_neighbor_cf,
                                                                               serialize(&to).unwrap())?
                                                               .unwrap()).unwrap();
                            for vote in votes {
                                added.push((vote, to_level));
                            }
                            to = deserialize(&self.db.get_cf(voter_parent_neighbor_cf,
                                                             serialize(&to).unwrap())?
                                             .unwrap()).unwrap();
                            to_level -= 1;
                        }

                        // trace back both from chain and to chain until they reach the same block
                        while to != from {
                            // trace back to chain
                            let votes: Vec<H256> = deserialize(&self.db.get_cf(vote_neighbor_cf,
                                                                               serialize(&to).unwrap())?
                                                               .unwrap()).unwrap();
                            for vote in votes {
                                added.push((vote, to_level));
                            }
                            to = deserialize(&self.db.get_cf(voter_parent_neighbor_cf,
                                                             serialize(&to).unwrap())?
                                             .unwrap()).unwrap();
                            to_level -= 1;

                            // trace back from chain
                            let votes: Vec<H256> = deserialize(&self.db.get_cf(vote_neighbor_cf,
                                                                               serialize(&from).unwrap())?
                                                               .unwrap()).unwrap();
                            for vote in votes {
                                removed.push((vote, from_level));
                            }
                            from = deserialize(&self.db.get_cf(voter_parent_neighbor_cf,
                                                               serialize(&from).unwrap())?
                                               .unwrap()).unwrap();
                            from_level -= 1;
                        }
                        for removed_vote in &removed {
                            wb.merge_cf(proposer_node_vote_cf, serialize(&removed_vote.0).unwrap(), serialize(&(false, self_chain as u16, removed_vote.1)).unwrap())?;
                        }
                        for added_vote in &added {
                            wb.merge_cf(proposer_node_vote_cf, serialize(&added_vote.0).unwrap(), serialize(&(true, self_chain as u16, added_vote.1)).unwrap())?;
                        }
                        // finally add the new votes in this new block
                        for added_vote in &content.votes {
                            wb.merge_cf(proposer_node_vote_cf, serialize(&added_vote).unwrap(), serialize(&(true, self_chain as u16, self_level as u64)).unwrap())?;
                        }
                    }
                }
            }
            Content::Transaction(_) => {
                // mark itself as unreferred
                let mut unreferred_transaction = self.unreferred_transaction.lock().unwrap();
                unreferred_transaction.insert(block_hash);
                drop(unreferred_transaction);
            }
        }
        self.db.write(wb)?;
        return Ok(());
    }

    pub fn best_proposer(&self) -> H256 {
        let proposer_best = self.proposer_best.lock().unwrap();
        let hash = proposer_best.0;
        drop(proposer_best);
        return hash;
    }

    pub fn best_voter(&self, chain_num: usize) -> H256 {
        let voter_best = self.voter_best[chain_num].lock().unwrap();
        let hash = voter_best.0;
        drop(voter_best);
        return hash;
    }

    pub fn unreferred_proposer(&self) -> Vec<H256> {
        // TODO: does ordering matter?
        // TODO: should remove the parent block when mining
        let unreferred_proposer = self.unreferred_proposer.lock().unwrap();
        let list: Vec<H256> = unreferred_proposer.iter().cloned().collect();
        drop(unreferred_proposer);
        return list;
    }

    pub fn unreferred_transaction(&self) -> Vec<H256> {
        // TODO: does ordering matter?
        let unreferred_transaction = self.unreferred_transaction.lock().unwrap();
        let list: Vec<H256> = unreferred_transaction.iter().cloned().collect();
        drop(unreferred_transaction);
        return list;
    }

    /// Get the list of unvoted proposer blocks that a voter chain should vote for, given the tip
    /// of the particular voter chain.
    pub fn unvoted_proposer(&self, tip: &H256) -> Result<Vec<H256>> {
        // get the deepest voted level
        let voter_node_voted_level_cf = self.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let voted_level: u64 = deserialize(&self.db.get_cf(voter_node_voted_level_cf,
                                                           serialize(&tip).unwrap())?
                                           .unwrap()).unwrap();
        // get the deepest proposer level
        let proposer_best = self.proposer_best.lock().unwrap();
        let proposer_best_level = proposer_best.1;
        drop(proposer_best);

        // get the first block we heard on each proposer level
        let mut list: Vec<H256> = vec![];
        for level in voted_level+1..=proposer_best_level {
            let blocks: Vec<H256> = deserialize(&self.db.get_cf(proposer_tree_level_cf, 
                                                     serialize(&(level as u64)).unwrap())?
                                     .unwrap()).unwrap();
            list.push(blocks[0]);
        }
        return Ok(list);
    }

    /// Get the hash of the leader block at the given level 
    pub fn proposer_leader(&self, level: u64) -> Result<Option<H256>> {
        let proposer_tree_level_cf = self.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let proposer_node_vote_cf = self.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();

        let proposer_blocks: Vec<H256> = match self.db.get_cf(proposer_tree_level_cf, serialize(&level).unwrap())? {
            None => return Ok(None),
            Some(d) => deserialize(&d).unwrap(),
        };

        for block in &proposer_blocks {
            let votes: Vec<(u16, u64)> = match self.db.get_cf(proposer_node_vote_cf, serialize(&block).unwrap())? {
                None => vec![],
                Some(d) => deserialize(&d).unwrap(),
            };

            if votes.len() as u16 > NUM_VOTER_CHAINS / 2 + 1 {
                return Ok(Some(*block));
            }
        }
        return Ok(None);
    }
}

fn vote_vec_merge(_: &[u8], existing_val: Option<&[u8]>, operands: &mut rocksdb::merge_operator::MergeOperands) -> Option<Vec<u8>> {
    let mut existing: Vec<(u16, u64)> = match existing_val {
        Some(v) => deserialize(v).unwrap(),
        None => vec![],
    };
    for op in operands {
        // parse the operation as add(true)/remove(false), chain(u16), level(u64)
        let operation: (bool, u16, u64) = deserialize(op).unwrap();
        match operation.0 {
            true => {
                existing.push((operation.1, operation.2));
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
        let proposer_node_vote_cf = db.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();
        let proposer_node_level_cf = db.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let voter_node_level_cf = db.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
        let voter_node_chain_cf = db.db.cf_handle(VOTER_NODE_CHAIN_CF).unwrap();
        let voter_node_voted_level_cf = db.db.cf_handle(VOTER_NODE_VOTED_LEVEL_CF).unwrap();
        let proposer_tree_level_cf = db.db.cf_handle(PROPOSER_TREE_LEVEL_CF).unwrap();
        let parent_neighbor_cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();
        let vote_neighbor_cf = db.db.cf_handle(VOTE_NEIGHBOR_CF).unwrap();
        let proposer_leader_sequence_cf = db.db.cf_handle(PROPOSER_LEADER_SEQUENCE_CF).unwrap();
        let proposer_confirm_list_cf = db.db.cf_handle(PROPOSER_CONFIRM_LIST_CF).unwrap();

        // validate proposer genesis
        let genesis_level: u64 = deserialize(&db.db.get_cf(proposer_node_level_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(genesis_level, 0);
        let level_0_blocks: Vec<H256> = deserialize(&db.db.get_cf(proposer_tree_level_cf, serialize(&(0 as u64)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level_0_blocks, vec![*PROPOSER_GENESIS_HASH]);
        let genesis_votes: Vec<(u16, u64)> = deserialize(&db.db.get_cf(proposer_node_vote_cf, serialize(&(*PROPOSER_GENESIS_HASH)).unwrap()).unwrap().unwrap()).unwrap();
        let mut true_genesis_votes: Vec<(u16, u64)> = vec![];
        for chain_num in 0..NUM_VOTER_CHAINS {
            true_genesis_votes.push((chain_num as u16, 0));
        }
        assert_eq!(genesis_votes, true_genesis_votes);
        assert_eq!(*db.proposer_best.lock().unwrap(), (*PROPOSER_GENESIS_HASH, 0));
        assert_eq!(db.unreferred_proposer.lock().unwrap().len(), 1);
        assert_eq!(db.unreferred_proposer.lock().unwrap().contains(&(PROPOSER_GENESIS_HASH)), true);
        let level_0_leader: H256 = deserialize(&db.db.get_cf(proposer_leader_sequence_cf, serialize(&(0 as u64)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level_0_leader, *PROPOSER_GENESIS_HASH);
        let level_0_confirms: Vec<H256> = deserialize(&db.db.get_cf(proposer_confirm_list_cf, serialize(&(0 as u64)).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level_0_confirms, vec![]);

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
    }

    #[test]
    fn insert_block() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_insert_block.rocksdb").unwrap();
        // get cf handles
        let proposer_node_vote_cf = db.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();
        let proposer_node_level_cf = db.db.cf_handle(PROPOSER_NODE_LEVEL_CF).unwrap();
        let voter_node_level_cf = db.db.cf_handle(VOTER_NODE_LEVEL_CF).unwrap();
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
        assert_eq!(db.unreferred_transaction.lock().unwrap().len(), 1);
        assert_eq!(db.unreferred_transaction.lock().unwrap().contains(&new_transaction_block.hash()), true);

        // Create two proposer blocks, both attached to the genesis proposer block. The first one
        // refers to nothing (except the parent), and the second one refers to the first one and
        // the transaction block
        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block_1 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [1; 32],
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
            [2; 32],
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
        assert_eq!(proposer_ref, vec![*PROPOSER_GENESIS_HASH, new_proposer_block_1.hash()]);
        let transaction_ref: Vec<H256> = deserialize(&db.db.get_cf(transaction_ref_neighbor_cf, serialize(&new_proposer_block_2.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(transaction_ref, vec![new_transaction_block.hash()]);
        assert_eq!(*db.proposer_best.lock().unwrap(), (new_proposer_block_1.hash(), 1));
        assert_eq!(db.unreferred_proposer.lock().unwrap().len(), 1);
        assert_eq!(db.unreferred_proposer.lock().unwrap().contains(&new_proposer_block_2.hash()), true);
        assert_eq!(db.unreferred_transaction.lock().unwrap().len(), 0);

        // Create a voter block attached to proposer block 2 and the first voter chain, and vote for proposer block 1.
        let new_voter_content = Content::Voter(voter::Content::new(0, VOTER_GENESIS_HASHES[0], vec![new_proposer_block_1.hash()]));
        let new_voter_block = Block::new(
            new_proposer_block_2.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [3; 32],
            H256::default()
        );
        db.insert_block(&new_voter_block).unwrap();

        let parent: H256 = deserialize(&db.db.get_cf(parent_neighbor_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(parent, new_proposer_block_2.hash());
        let voter_parent: H256 = deserialize(&db.db.get_cf(voter_parent_neighbor_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(voter_parent, VOTER_GENESIS_HASHES[0]);
        let level: u64 = deserialize(&db.db.get_cf(voter_node_level_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(level, 1);
        let chain: u16 = deserialize(&db.db.get_cf(voter_node_chain_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(chain, 0);
        let voted_level: u64 = deserialize(&db.db.get_cf(voter_node_voted_level_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(voted_level, 1);
        let voted: Vec<H256> = deserialize(&db.db.get_cf(vote_neighbor_cf, serialize(&new_voter_block.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(voted, vec![new_proposer_block_1.hash()]);
        assert_eq!(*db.voter_best[0].lock().unwrap(), (new_voter_block.hash(), 1));
        let votes: Vec<(u16, u64)> = deserialize(&db.db.get_cf(proposer_node_vote_cf, serialize(&new_proposer_block_1.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(votes, vec![(0, 1)]);

        // Create a fork of the voter chain and vote for proposer block 2.
        let new_voter_content = Content::Voter(voter::Content::new(0, VOTER_GENESIS_HASHES[0], vec![new_proposer_block_2.hash()]));
        let new_voter_block = Block::new(
            new_proposer_block_2.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [4; 32],
            H256::default()
        );
        db.insert_block(&new_voter_block).unwrap();
        let votes: Vec<(u16, u64)> = deserialize(&db.db.get_cf(proposer_node_vote_cf, serialize(&new_proposer_block_1.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(votes, vec![(0, 1)]);
        
        // Add to this fork, so that it becomes the longest chain.
        let new_voter_content = Content::Voter(voter::Content::new(0, new_voter_block.hash(), vec![]));
        let new_voter_block = Block::new(
            new_proposer_block_2.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [5; 32],
            H256::default()
        );
        db.insert_block(&new_voter_block).unwrap();

        let votes: Vec<(u16, u64)> = deserialize(&db.db.get_cf(proposer_node_vote_cf, serialize(&new_proposer_block_1.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(votes, vec![]);
        let votes: Vec<(u16, u64)> = deserialize(&db.db.get_cf(proposer_node_vote_cf, serialize(&new_proposer_block_2.hash()).unwrap()).unwrap().unwrap()).unwrap();
        assert_eq!(votes, vec![(0, 1)]);
    }

    #[test]
    fn proposer_leader() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_proposer_leader.rocksdb").unwrap();
        assert_eq!(db.proposer_leader(0).unwrap().unwrap(), *PROPOSER_GENESIS_HASH);

        // Insert two proposer blocks so we have something to vote for
        let proposer_1 = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let proposer_1 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            proposer_1,
            [255; 32],
            H256::default()
        );
        db.insert_block(&proposer_1).unwrap();
        let proposer_2 = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let proposer_2 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            proposer_2,
            [254; 32],
            H256::default()
        );
        db.insert_block(&proposer_2).unwrap();
        
        // For each voter chain, insert a voter block to vote for proposer 1. Check that only after
        // we get more than half chains voting for it do we get a leader
        for chain_num in 0..NUM_VOTER_CHAINS {
            let voter = Content::Voter(voter::Content::new(0, VOTER_GENESIS_HASHES[chain_num as usize], vec![proposer_1.hash()]));
            let voter = Block::new(
                proposer_1.hash(), 
                0,
                0,
                H256::default(),
                vec![],
                voter,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ((chain_num >> 8) & 0xff) as u8, (chain_num & 0xff) as u8],
                H256::default()
            );
            db.insert_block(&voter).unwrap();
            let leader = db.proposer_leader(1).unwrap();
            if chain_num + 1 as u16 <= NUM_VOTER_CHAINS / 2 + 1 {
                assert_eq!(leader, None);
            }
            else {
                println!("{}", chain_num + 1);
                assert_eq!(leader, Some(proposer_1.hash()));
            }
        }
    }

    #[test]
    fn best_proposer_and_voter() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_best_proposer_and_voter.rocksdb").unwrap();
        assert_eq!(db.best_proposer(), *PROPOSER_GENESIS_HASH);
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
            H256::default()
        );
        db.insert_block(&new_proposer_block).unwrap();
        let new_voter_content = Content::Voter(voter::Content::new(0, VOTER_GENESIS_HASHES[0], vec![new_proposer_block.hash()]));
        let new_voter_block = Block::new(
            new_proposer_block.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [1; 32],
            H256::default()
        );
        db.insert_block(&new_voter_block).unwrap();
        assert_eq!(db.best_proposer(), new_proposer_block.hash());
        assert_eq!(db.best_voter(0), new_voter_block.hash());
    }

    #[test]
    fn unreferred_transaction_and_proposer() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_unreferred_transaction_proposer.rocksdb").unwrap();
        
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
        assert_eq!(db.unreferred_transaction(), vec![new_transaction_block.hash()]);
        assert_eq!(db.unreferred_proposer(), vec![*PROPOSER_GENESIS_HASH]);

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block_1 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [1; 32],
            H256::default()
        );
        db.insert_block(&new_proposer_block_1).unwrap();
        assert_eq!(db.unreferred_transaction(), vec![new_transaction_block.hash()]);
        assert_eq!(db.unreferred_proposer(), vec![new_proposer_block_1.hash()]);

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![new_transaction_block.hash()], vec![new_proposer_block_1.hash()]));
        let new_proposer_block_2 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [2; 32],
            H256::default()
        );
        db.insert_block(&new_proposer_block_2).unwrap();
        assert_eq!(db.unreferred_transaction(), vec![]);
        assert_eq!(db.unreferred_proposer(), vec![new_proposer_block_2.hash()]);
    }

    #[test]
    fn unvoted_proposer() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_unvoted_proposer.rocksdb").unwrap();
        assert_eq!(db.unvoted_proposer(&VOTER_GENESIS_HASHES[0]).unwrap(), vec![]);

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

        let new_proposer_content = Content::Proposer(proposer::Content::new(vec![], vec![]));
        let new_proposer_block_2 = Block::new(
            *PROPOSER_GENESIS_HASH,
            0,
            0,
            H256::default(),
            vec![],
            new_proposer_content,
            [1; 32],
            H256::default()
        );
        db.insert_block(&new_proposer_block_2).unwrap();
        assert_eq!(db.unvoted_proposer(&VOTER_GENESIS_HASHES[0]).unwrap(), vec![new_proposer_block_1.hash()]);

        let new_voter_content = Content::Voter(voter::Content::new(0, VOTER_GENESIS_HASHES[0], vec![new_proposer_block_1.hash()]));
        let new_voter_block = Block::new(
            new_proposer_block_2.hash(),
            0,
            0,
            H256::default(),
            vec![],
            new_voter_content,
            [2; 32],
            H256::default()
        );
        db.insert_block(&new_voter_block).unwrap();

        assert_eq!(db.unvoted_proposer(&VOTER_GENESIS_HASHES[0]).unwrap(), vec![new_proposer_block_1.hash()]);
        assert_eq!(db.unvoted_proposer(&new_voter_block.hash()).unwrap(), vec![]);
    }

    #[test]
    fn merge_operator_h256_vec() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_merge_op_h256_vec.rocksdb").unwrap();
        let cf = db.db.cf_handle(PARENT_NEIGHBOR_CF).unwrap();

        // merge with an nonexistent entry
        db.db.merge_cf(cf, b"testkey", serialize(&H256::default()).unwrap()).unwrap();
        let result: Vec<H256> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![H256::default()]);

        // merge with an existing entry
        db.db.merge_cf(cf, b"testkey", serialize(&H256::default()).unwrap()).unwrap();
        db.db.merge_cf(cf, b"testkey", serialize(&H256::default()).unwrap()).unwrap();
        let result: Vec<H256> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![H256::default(), H256::default(), H256::default()]);
    }

    #[test]
    fn merge_operator_btreemap() {
        let db = BlockChain::new("/tmp/prism_test_blockchain_merge_op_u64_vec.rocksdb").unwrap();
        let cf = db.db.cf_handle(PROPOSER_NODE_VOTE_CF).unwrap();

        // merge with an nonexistent entry
        db.db.merge_cf(cf, b"testkey", serialize(&(true, 0 as u16, 0 as u64)).unwrap()).unwrap();
        let result: Vec<(u16, u64)> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![(0, 0)]);

        // insert
        db.db.merge_cf(cf, b"testkey", serialize(&(true, 10 as u16, 0 as u64)).unwrap()).unwrap();
        db.db.merge_cf(cf, b"testkey", serialize(&(true, 5 as u16, 0 as u64)).unwrap()).unwrap();
        let result: Vec<(u16, u64)> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![(0, 0), (10, 0), (5, 0)]);

        // remove
        db.db.merge_cf(cf, b"testkey", serialize(&(false, 5 as u16, 0 as u64)).unwrap()).unwrap();
        let result: Vec<(u16, u64)> = deserialize(&db.db.get_cf(cf, b"testkey").unwrap().unwrap()).unwrap();
        assert_eq!(result, vec![(0, 0), (10, 0)]);
    }
}

