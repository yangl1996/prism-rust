// Draft of a set of stateless APIs to confirm proposer blocks.
//
// ----- API Specification -----
//
// Calculate the leader of a proposer level (list confirmation).
// <- Votes of each voter chain on this proposer level: depth, and vote for whom
// -> A list of confirmed proposer blocks
//
// Get the vote of a voter chain on a proposer level
// <- An identifier of the voter chain (not just the ID but which chain)
// <- The proposer level that we are interested in
// -> The proposer block the chain votes for
// -> The depth of the vote
//
// The data structures returned and supplied in those API calls must be functional.
//
// ----- Implementation Draft -----
//
//            A
//            |
//            B
//            | \
//            C  D
//            |  | \
//            E  F  G
//
// Keeping a record of which voter chain votes for which proposer block on each level
// - Chop a voter tree into "segments" - longest portions that have no fork
// - Identify a "segment" by its starting and ending block
//   - A segment begins with a block with 0 or 1 child, and ends with a block with 0 or 2+ children
//   - For example, AB, CE, D, F, G each is a segment
// - For each segment, we store which proposer block it votes on each level
// - Note that a segment may be broken apart (in the case of forking), or be extended
// - In the API, identify a segment by the lowest and highest proposer level it votes
//
// ----- Confirmation Logic -----
//
// Voter chain tip -> Vote and depth --x1000--> Proposer leader
// Proposer level  -> 
//
// When a new voter block arrives:
// - Extends the main chain: no existing ledger changes except list confirmation
// - Extends a fork: no existing ledger changes or new confirmation
// - Switches the main chain: everything up to the fork does not change
//
// Every confirmed (not list confirmed) proposer block should have a signature recording the
// condition on which it is confirmed: The segment IDs of each voter chain. Then we simply compare
// the current chain state with when it is confirmed. If the current state has the confirmation
// state as a prefix, then we don't need to do anything. Otherwise, recompute the proposer leader.
use crate::crypto::hash::H256;
use std::sync::{Arc, Weak};
use std::convert::TryFrom;
use crate::block::Block as RealBlock;
use crate::block::Content;

//         Segment 1
// /                      \ 
// ------------------------
//          \
//           -----------
//           \         /
//             Sgmt. 2
//
// Segment 2 has parent Segment 1.
//
// Idea: we should store the trees top-down, so that we can simply drop the root to shrink the size
// of the tree for garbage collection.
pub struct Segment {
    pub lowest_vote_level: u64,     // inclusive
    pub highest_vote_level: u64,    // inclusive
    pub lowest_block: (H256, u64),
    pub highest_block: (H256, u64),
    pub parent: Option<Arc<Segment>>,
    pub votes: Vec<(H256, u64)>,    // proposer hash, level of the vote
}

pub trait Block {
    type Ref;   // can't make Ref = &[something] here due to lack of GAT in rust

    fn attach(self: &Arc<Self>, hash: H256, refs: &[Self::Ref]) -> Self;

    fn level(&self) -> u64;

    fn hash(&self) -> H256;
}

pub struct Proposer {
    pub level: u64,
    pub hash: H256,
    pub tx_refs: Vec<H256>,
    pub prop_refs: Vec<Weak<Proposer>>,
    pub parent: Weak<Proposer>,
}

pub enum ProposerReference {
    Proposer(Weak<Proposer>),
    Transaction(H256),
}

impl Block for Proposer {
    type Ref = ProposerReference;

    fn attach(self: &Arc<Self>, hash: H256, refs: &[ProposerReference]) -> Self {
        let mut tx_refs = vec![];
        let mut prop_refs = vec![];
        for r in refs.iter() {
            match r {
                ProposerReference::Proposer(ptr) => prop_refs.push(Weak::clone(&ptr)),
                ProposerReference::Transaction(h) => tx_refs.push(*h),
            }
        }
        Self {
            level: self.level + 1,
            hash,
            tx_refs,
            prop_refs,
            parent: Arc::downgrade(self),
        }
    }

    fn level(&self) -> u64 {
        self.level
    }

    fn hash(&self) -> H256 {
        self.hash
    }
}

pub struct Voter {
    pub level: u64,
    pub hash: H256,
    pub vote_start_level: u64,      // inclusive
    pub votes: Vec<H256>,           // hashes of proposer blocks voted, organized by level
    pub parent: Weak<Voter>,        // using Weak to allow garbage collection
}

impl Voter {
    /// Given the voter chain (identified by its tip voter block) and the proposer level we are
    /// interested in, get the proposer block on that level voted by this voter chain and the depth of
    /// the vote.
    pub fn proposer_vote_of_level(&self, proposer_level: u64) -> Option<(H256, u64)> {
        // For now, we simply do a linear search. TODO: implement a functional segment tree to improve
        // the speed that we search the segment containing a specific proposer level.

        let best_voter_level = self.level;
        // First check the current voter block
        if proposer_level >= self.vote_start_level + u64::try_from(self.votes.len()).unwrap() {
            // The proposer level is higher than the highest voted level of this chain
            return None;
        }
        else {
            if proposer_level >= self.vote_start_level {
                // This voter block votes for this proposer level
                let idx = usize::try_from(proposer_level - self.vote_start_level).unwrap();
                let vote = self.votes[idx];
                return Some((vote, 1));
            }
        }

        // Then trace back
        let mut current_block = match self.parent.upgrade() {
            Some(p) => p,
            None => return None,
        };
        while proposer_level < current_block.vote_start_level + u64::try_from(current_block.votes.len()).unwrap() {
            if proposer_level >= current_block.vote_start_level {
                let idx = usize::try_from(proposer_level - current_block.vote_start_level).unwrap();
                let vote = current_block.votes[idx];
                return Some((vote, best_voter_level - current_block.level + 1));
            }
            else {
                current_block = match current_block.parent.upgrade() {
                    Some(p) => p,
                    None => return None,
                };
            }
        }
        None
    }
}

impl Block for Voter {
    type Ref = H256; // holding the proposer blocks to vote for

    fn attach(self: &Arc<Self>, hash: H256, refs: &[H256]) -> Self {
        Self {
            level: self.level + 1,
            hash,
            vote_start_level: self.vote_start_level + u64::try_from(self.votes.len()).unwrap(),
            votes: refs.to_vec(),
            parent: Arc::downgrade(self),
        }
    }

    fn level(&self) -> u64 {
        self.level
    }

    fn hash(&self) -> H256 {
        self.hash
    }
}

//        -----      ------
//       /          /
// -------------------------------
//                \
//                 --------
// For now, we set a threshold level and remove everything lower than that level

pub struct ChainIndex<B: Block> {
    blocks: std::collections::HashMap<H256, Arc<B>>,
    starting_level: u64,                                    // the level stored by index 0 in the vecdeque
    by_level: std::collections::VecDeque<Vec<H256>>,        // organized by level in increasing order
}

impl<B: Block> ChainIndex<B> {
    pub fn new() -> Self {
        Self {
            blocks: std::collections::HashMap::new(),
            starting_level: 0,
            by_level: std::collections::VecDeque::new(),
        }
    }

    // TODO: looks like it can be optimized with a segment tree
    pub fn num_blocks(&self, start: u64, end: u64) -> usize {
        if start < self.starting_level {
            panic!("Counting blocks beginning at a level lower than the chain index starting level");
        }
        let start_idx = usize::try_from(start - self.starting_level).unwrap();
        let count = usize::try_from(end - self.starting_level).unwrap() + 1; 
        let count = if count > self.by_level.len() {
            self.by_level.len()
        } else {
            count
        };
        let mut total = 0;
        for i in start_idx..count {
            total += self.by_level[i].len();
        }
        return total;
    }

    pub fn highest_block(&self) -> Arc<B> {
        if self.by_level.is_empty() {
            panic!("Querying the highest block from an empty chain index");
        }
        let highest_level = self.by_level.back().unwrap();
        if highest_level.is_empty() {
            panic!("The highest level of the chain index is empty");
        }
        let block_hash = highest_level[0];
        match self.blocks.get(&block_hash) {
            Some(p) => return Arc::clone(&p),
            None => panic!("Hash stored on the highest level does not exist in the hashmap"),
        }
    }

    pub fn remove_levels(&mut self, new_start_level: u64) {
        if new_start_level <= self.starting_level {
            return;
        }
        let levels_to_remove = usize::try_from(new_start_level - self.starting_level).unwrap();
        if levels_to_remove >= self.by_level.len() {
            panic!("Removing all levels in the chain index");
        }
        let new = self.by_level.split_off(levels_to_remove);
        for l in self.by_level.iter() {
            for h in l.iter() {
                if self.blocks.remove(&h).is_none() {
                    panic!("Block hash exists in the by-level list but not in the hashtable");
                }
            }
        }
        self.by_level = new;
        self.starting_level = new_start_level;
    }

    fn insert_at_root(&mut self, block: &Arc<B>) {
        let block = Arc::clone(block);
        let block_empty = self.blocks.is_empty();
        let level_empty = self.by_level.is_empty();

        if block_empty && !level_empty {
            panic!("Chain index has zero block but non-zero level");
        }
        else if block_empty && level_empty {
            // if the index is previously empty
            self.starting_level = block.level();
            self.blocks.insert(block.hash(), Arc::clone(&block));
            self.by_level.push_back(vec![block.hash()]);
        }
        else if (!block_empty) && level_empty {
            panic!("Chain index has non-zero blocks but zero level");
        }
        else {
            // if the index is nonempty
            if block.level() != self.starting_level {
                panic!("Adding a root at a level different from the index starting level");
            }
            let list = self.by_level.get_mut(0).unwrap();
            if list.contains(&block.hash()) {
                panic!("Adding a root already there on that level");
            }
            list.push(block.hash());
            self.blocks.insert(block.hash(), Arc::clone(&block));
        }
    }

    fn insert_block(&mut self, block: &Arc<B>) {
        let level = block.level();
        self.blocks.insert(block.hash(), Arc::clone(block));
        let level_idx = level - self.starting_level;
        let vec_len = u64::try_from(self.by_level.len()).unwrap();
        if level_idx > vec_len {
            panic!("Adding a block deeper than the current deepest level + 1");
        }
        else if level_idx == vec_len {
            self.by_level.push_back(vec![block.hash()]);
        }
        else {
            let list = self.by_level.get_mut(usize::try_from(level_idx).unwrap()).unwrap();
            if list.contains(&block.hash()) {
                panic!("Adding a block already there on that level");
            }
            list.push(block.hash());
        }


    }
}

impl ChainIndex<Proposer> {
    pub fn insert_proposer_root_at(&mut self, block: &RealBlock, hash: H256, level: u64) -> Arc<Proposer> {
        let content = match &block.content {
            Content::Proposer(stuff) => stuff,
            _ => panic!("Adding a non-proposer block to a proposer chain as a root"),
        };

        let block = Proposer {
            level,
            hash,
            tx_refs: content.transaction_refs.to_vec(),
            prop_refs: vec![],
            parent: Default::default(),
        };
        let block = Arc::new(block);

        self.insert_at_root(&block);
        return block;
    }

    pub fn insert_proposer(&mut self, block: &RealBlock, hash: H256) -> Arc<Proposer> {
        let content = match &block.content {
            Content::Proposer(stuff) => stuff,
            _ => panic!("Adding a non-proposer block to a proposer chain"),
        };
        let parent = block.header.parent;
        let parent_ref = match self.blocks.get(&parent) {
            Some(v) => v,
            None => panic!("Adding a proposer block whose parent is unknown"),
        };
        let mut refs = vec![];
        for tref in content.transaction_refs.iter() {
            refs.push(ProposerReference::Transaction(*tref));
        }
        for pref in content.proposer_refs.iter() {
            if let Some(ptr) = self.blocks.get(&pref) {
                refs.push(ProposerReference::Proposer(Arc::downgrade(&ptr)));
            }
            else {
                panic!("Adding a proposer which refers to a proposer block not in the index");
            }
        }
        
        let new_block = parent_ref.attach(hash, &refs);
        let new_block = Arc::new(new_block);
        self.insert_block(&new_block);
        return new_block;
    }
}

impl ChainIndex<Voter> {
    pub fn insert_voter_root_at(&mut self, block: &RealBlock, hash: H256, level: u64, vote_start_level: u64) -> Arc<Voter> {
        let content = match &block.content {
            Content::Voter(stuff) => stuff,
            _ => panic!("Adding a non-voter block to a voter chain as a root"),
        };

        let voter = Voter {
            level,
            hash,
            vote_start_level,
            votes: content.votes.to_vec(),
            parent: Default::default(),
        };
        let voter = Arc::new(voter);

        self.insert_at_root(&voter);
        return voter;
    }

    pub fn insert_voter(&mut self, block: &RealBlock, hash: H256) -> Arc<Voter> {
        let content = match &block.content {
            Content::Voter(stuff) => stuff,
            _ => panic!("Adding a non-voter block to a voter chain"),
        };
        let parent = content.voter_parent;
        let parent_ref = match self.blocks.get(&parent) {
            Some(v) => v,
            None => panic!("Adding a voter block whose parent is unknown"),
        };
        
        let new_block = parent_ref.attach(hash, &content.votes);
        let new_block = Arc::new(new_block);
        self.insert_block(&new_block);
        return new_block;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use crate::block::{Block as RealBlock, voter::Content as VoterContent, header::Header, proposer::Content as ProposerContent};
    use crate::crypto::hash::Hashable;

    fn get_hash() -> H256 {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen_range(0, 255) as u8).collect();
        let mut raw_bytes = [0; 32];
        raw_bytes.copy_from_slice(&random_bytes);
        (&raw_bytes).into()
    }

    fn cmp(a: &Voter, b: &Voter) -> bool {
            if a.level != b.level {
                return false;
            }
            if a.hash != b.hash {
                return false;
            }
            if a.vote_start_level != b.vote_start_level {
                return false;
            }
            if a.votes != b.votes {
                return false;
            }
            let ap = a.parent.upgrade();
            let bp = b.parent.upgrade();
            if ap.is_none() && bp.is_none() {
                return true;
            } else {
                if ap.is_some() && bp.is_some() {
                    return cmp(&ap.unwrap(), &bp.unwrap());
                }
                else {
                    return false;
                }
            }
    }

    fn voter_block(parent: H256, votes: &[H256]) -> RealBlock {
        RealBlock {
            header: Header {
                parent: get_hash(),
                timestamp: 0,
                nonce: 0,
                content_merkle_root: get_hash(),
                extra_content: [0; 32],
                difficulty: get_hash(),
            },
            content: Content::Voter(VoterContent {
                chain_number: 0,
                voter_parent: parent,
                votes: votes.to_vec(),
            }),
            sortition_proof: vec![],
        }
    }

    fn proposer_block(parent: H256, prefs: &[H256], trefs: &[H256]) -> RealBlock {
        RealBlock {
            header: Header {
                parent: parent,
                timestamp: 0,
                nonce: 0,
                content_merkle_root: get_hash(),
                extra_content: [0; 32],
                difficulty: get_hash(),
            },
            content: Content::Proposer(ProposerContent {
                proposer_refs: prefs.to_vec(),
                transaction_refs: trefs.to_vec(),
            }),
            sortition_proof: vec![],
        }
    }

    #[test]
    fn proposer_index() {
        // level starts at 15
        // p0 - p1 - p3 - p5
        //   \- p2 - p4
        // 
        // p3 refers to p2, p4 refers to p1, p5 refers to p4
        let tx_ref1 = get_hash();
        let tx_ref2 = get_hash();
        let p0 = proposer_block(get_hash(), &[], &[tx_ref1, get_hash()]);
        let p1 = proposer_block(p0.hash(), &[], &[get_hash()]);
        let p2 = proposer_block(p0.hash(), &[], &[get_hash(), get_hash()]);
        let p3 = proposer_block(p1.hash(), &[p2.hash()], &[get_hash(), get_hash()]);
        let p4 = proposer_block(p2.hash(), &[p1.hash()], &[tx_ref2]);
        let p5 = proposer_block(p3.hash(), &[p4.hash()], &[get_hash()]);
        
        let mut prop_blocks = vec![];
        let mut idx = ChainIndex::new();
        prop_blocks.push(idx.insert_proposer_root_at(&p0, p0.hash(), 15));
        prop_blocks.push(idx.insert_proposer(&p1, p1.hash()));
        prop_blocks.push(idx.insert_proposer(&p2, p2.hash()));
        prop_blocks.push(idx.insert_proposer(&p3, p3.hash()));
        prop_blocks.push(idx.insert_proposer(&p4, p4.hash()));
        prop_blocks.push(idx.insert_proposer(&p5, p5.hash()));

        // p5 refers to p4
        assert!(Arc::ptr_eq(&prop_blocks[4], &prop_blocks[5].prop_refs[0].upgrade().unwrap()));
        assert_eq!(prop_blocks[5].prop_refs.len(), 1);
        // p4 refers to p1
        assert!(Arc::ptr_eq(&prop_blocks[1], &prop_blocks[4].prop_refs[0].upgrade().unwrap()));
        assert_eq!(prop_blocks[4].prop_refs.len(), 1);
        // p3 refers to p2
        assert!(Arc::ptr_eq(&prop_blocks[2], &prop_blocks[3].prop_refs[0].upgrade().unwrap()));
        assert_eq!(prop_blocks[3].prop_refs.len(), 1);
        assert!(prop_blocks[1].prop_refs.is_empty());
        // p0 tx refs
        assert_eq!(&tx_ref1, &prop_blocks[0].tx_refs[0]);
        assert_eq!(prop_blocks[0].tx_refs.len(), 2);
        assert_eq!(&tx_ref2, &prop_blocks[4].tx_refs[0]);
        assert_eq!(prop_blocks[4].tx_refs.len(), 1);
    }

    #[test]
    fn remove_level() {
        // level starts at 30
        // v0 - v1 - v2 - v3
        //        \- v4
        let mut voter_blocks: Vec<RealBlock> = vec![];
        let b0 = voter_block(get_hash(), &[]);
        voter_blocks.push(b0);
        let b1 = voter_block(voter_blocks[0].hash(), &[]);
        voter_blocks.push(b1);
        let b2 = voter_block(voter_blocks[1].hash(), &[]);
        voter_blocks.push(b2);
        let b3 = voter_block(voter_blocks[2].hash(), &[]);
        voter_blocks.push(b3);
        let b4 = voter_block(voter_blocks[1].hash(), &[]);
        voter_blocks.push(b4);

        let mut idx = ChainIndex::new();
        idx.insert_voter_root_at(&voter_blocks[0], voter_blocks[0].hash(), 30, 10);
        idx.insert_voter(&voter_blocks[1], voter_blocks[1].hash());
        idx.insert_voter(&voter_blocks[2], voter_blocks[2].hash());
        idx.insert_voter(&voter_blocks[3], voter_blocks[3].hash());
        idx.insert_voter(&voter_blocks[4], voter_blocks[4].hash());

        assert_eq!(idx.starting_level, 30);
        assert_eq!(idx.by_level.len(), 4);
        idx.remove_levels(20);
        assert_eq!(idx.starting_level, 30);
        assert_eq!(idx.by_level.len(), 4);
        idx.remove_levels(30);
        assert_eq!(idx.starting_level, 30);
        assert_eq!(idx.by_level.len(), 4);
        idx.remove_levels(32);
        assert_eq!(idx.starting_level, 32);
        assert_eq!(idx.by_level.len(), 2);
        assert!(idx.by_level[0].contains(&voter_blocks[2].hash()));
        assert!(idx.by_level[0].contains(&voter_blocks[4].hash()));
        assert_eq!(idx.blocks.len(), 3);
        idx.remove_levels(33);
        assert_eq!(idx.starting_level, 33);
        assert_eq!(idx.by_level.len(), 1);
        assert!(idx.by_level[0].contains(&voter_blocks[3].hash()));
        assert_eq!(idx.blocks.len(), 1);
    }

    #[test]
    fn insert_voter_block() {
        let mut proposer_blocks: Vec<H256> = vec![];
        // level starts at 10
        // p0 - p1 - p2 - p3
        //             \- p4
        //
        // level starts at 30
        // v0 - v1 - v2 - v3
        //        \- v4
        //
        // v0 -> p0, p1; v1 -> p2; v2 -> []; v3 -> p3; v4 -> p4
        for _ in 0..5 {
            proposer_blocks.push(get_hash());
        }
        let mut voter_blocks: Vec<RealBlock> = vec![];
        let b0 = voter_block(get_hash(), &[proposer_blocks[0], proposer_blocks[1]]);
        voter_blocks.push(b0);
        let b1 = voter_block(voter_blocks[0].hash(), &[proposer_blocks[2]]);
        voter_blocks.push(b1);
        let b2 = voter_block(voter_blocks[1].hash(), &[]);
        voter_blocks.push(b2);
        let b3 = voter_block(voter_blocks[2].hash(), &[proposer_blocks[3]]);
        voter_blocks.push(b3);
        let b4 = voter_block(voter_blocks[1].hash(), &[proposer_blocks[4]]);
        voter_blocks.push(b4);

        let mut idx = ChainIndex::new();
        idx.insert_voter_root_at(&voter_blocks[0], voter_blocks[0].hash(), 30, 10);
        let b = idx.insert_voter(&voter_blocks[1], voter_blocks[1].hash());
        assert!(cmp(&idx.highest_block(), &b));
        let b = idx.insert_voter(&voter_blocks[2], voter_blocks[2].hash());
        assert!(cmp(&idx.highest_block(), &b));
        let tip2 = idx.insert_voter(&voter_blocks[3], voter_blocks[3].hash());
        assert!(cmp(&idx.highest_block(), &tip2));
        let tip = idx.insert_voter(&voter_blocks[4], voter_blocks[4].hash());
        assert!(cmp(&idx.highest_block(), &tip2));
        assert_eq!(tip.proposer_vote_of_level(9), None);
        assert_eq!(tip.proposer_vote_of_level(10), Some((proposer_blocks[0], 3)));
        assert_eq!(tip.proposer_vote_of_level(11), Some((proposer_blocks[1], 3)));
        assert_eq!(tip.proposer_vote_of_level(12), Some((proposer_blocks[2], 2)));
        assert_eq!(tip.proposer_vote_of_level(13), Some((proposer_blocks[4], 1)));
        assert_eq!(tip.proposer_vote_of_level(14), None);
        assert_eq!(tip2.proposer_vote_of_level(9), None);
        assert_eq!(tip2.proposer_vote_of_level(10), Some((proposer_blocks[0], 4)));
        assert_eq!(tip2.proposer_vote_of_level(11), Some((proposer_blocks[1], 4)));
        assert_eq!(tip2.proposer_vote_of_level(12), Some((proposer_blocks[2], 3)));
        assert_eq!(tip2.proposer_vote_of_level(13), Some((proposer_blocks[3], 1)));
        assert_eq!(tip2.proposer_vote_of_level(14), None);
        assert_eq!(idx.by_level[0].len(), 1);
        assert_eq!(idx.by_level[1].len(), 1);
        assert_eq!(idx.by_level[2].len(), 2);
        assert_eq!(idx.by_level[3].len(), 1);
        assert!(idx.by_level[0].contains(&voter_blocks[0].hash()));
        assert!(idx.by_level[1].contains(&voter_blocks[1].hash()));
        assert!(idx.by_level[2].contains(&voter_blocks[2].hash()));
        assert!(idx.by_level[2].contains(&voter_blocks[4].hash()));
        assert!(idx.by_level[3].contains(&voter_blocks[3].hash()));
        assert_eq!(idx.num_blocks(60, 62), 0);
        assert_eq!(idx.num_blocks(30, 38), 5);
        assert_eq!(idx.num_blocks(30, 33), 5);
        assert_eq!(idx.num_blocks(30, 31), 2);
        assert_eq!(idx.num_blocks(32, 32), 2);
        assert_eq!(idx.num_blocks(31, 33), 4);
    }

    #[test]
    fn attach_new_voter() {
        // the groundtruth
        let mut proposer_blocks: Vec<H256> = vec![];
        for _ in 0..7 {
            proposer_blocks.push(get_hash());
        }
        let mut voter_blocks: Vec<H256> = vec![];
        for _ in 0..7 {
            voter_blocks.push(get_hash());
        }

        let mut voters: Vec<Arc<Voter>> = vec![];
        let mut this_vote_level = 0;
        let mut this_level = 0;
        let mut last_voter_block: Option<Arc<Voter>> = None;
        let mut create_voter = |votes: Vec<H256>| -> Arc<Voter> {
            let parent_ref = match &last_voter_block {
                Some(p) => Arc::downgrade(&p),
                None => Default::default(),
            };
            let v = Voter {
                level: this_level,
                hash: voter_blocks[this_level as usize],
                vote_start_level: this_vote_level,
                votes: votes,
                parent: parent_ref,
            };
            let v = Arc::new(v);
            voters.push(Arc::clone(&v));
            this_vote_level += v.votes.len() as u64;
            this_level += 1;
            last_voter_block = Some(Arc::clone(&v));
            return v;
        };

        create_voter(vec![proposer_blocks[0]]);
        create_voter(vec![]);
        create_voter(vec![proposer_blocks[1], proposer_blocks[2]]);
        create_voter(vec![proposer_blocks[3]]);
        create_voter(vec![proposer_blocks[4]]);
        create_voter(vec![]);
        create_voter(vec![proposer_blocks[5], proposer_blocks[6]]);

        // results from the function
        let mut my_voters: Vec<Arc<Voter>> = vec![];
        my_voters.push(Arc::clone(&voters[0]));
        let v = my_voters[0].attach(voter_blocks[1], &vec![]);
        my_voters.push(Arc::new(v));
        let v = my_voters[1].attach(voter_blocks[2], &vec![proposer_blocks[1], proposer_blocks[2]]);
        my_voters.push(Arc::new(v));
        let v = my_voters[2].attach(voter_blocks[3], &vec![proposer_blocks[3]]);
        my_voters.push(Arc::new(v));
        let v = my_voters[3].attach(voter_blocks[4], &vec![proposer_blocks[4]]);
        my_voters.push(Arc::new(v));
        let v = my_voters[4].attach(voter_blocks[5], &vec![]);
        my_voters.push(Arc::new(v));
        let v = my_voters[5].attach(voter_blocks[6], &vec![proposer_blocks[5], proposer_blocks[6]]);
        my_voters.push(Arc::new(v));

        assert!(cmp(&voters[6], &my_voters[6]));
    }

    #[test]
    fn get_proposer_vote_by_level() {
        // generate 7 hashes for proposer blocks, and 7 hashes for voter blocks
        let mut proposer_blocks: Vec<H256> = vec![];
        for _ in 0..7 {
            proposer_blocks.push(get_hash());
        }
        let mut voter_blocks: Vec<H256> = vec![];
        for _ in 0..7 {
            voter_blocks.push(get_hash());
        }

        let mut voters: Vec<Arc<Voter>> = vec![];
        let mut this_vote_level = 0;
        let mut this_level = 0;
        let mut last_voter_block: Option<Arc<Voter>> = None;
        let mut create_voter = |votes: Vec<H256>| -> Arc<Voter> {
            let parent_ref = match &last_voter_block {
                Some(p) => Arc::downgrade(&p),
                None => Default::default(),
            };
            let v = Voter {
                level: this_level,
                hash: voter_blocks[this_level as usize],
                vote_start_level: this_vote_level,
                votes: votes,
                parent: parent_ref,
            };
            let v = Arc::new(v);
            voters.push(Arc::clone(&v));
            this_vote_level += v.votes.len() as u64;
            this_level += 1;
            last_voter_block = Some(Arc::clone(&v));
            return v;
        };

        create_voter(vec![proposer_blocks[0]]);
        create_voter(vec![]);
        create_voter(vec![proposer_blocks[1], proposer_blocks[2]]);
        create_voter(vec![proposer_blocks[3]]);
        create_voter(vec![proposer_blocks[4]]);
        create_voter(vec![]);
        let segment = create_voter(vec![proposer_blocks[5], proposer_blocks[6]]);

        assert_eq!(segment.proposer_vote_of_level(0), Some((proposer_blocks[0], 7)));
        assert_eq!(segment.proposer_vote_of_level(1), Some((proposer_blocks[1], 5)));
        assert_eq!(segment.proposer_vote_of_level(2), Some((proposer_blocks[2], 5)));
        assert_eq!(segment.proposer_vote_of_level(3), Some((proposer_blocks[3], 4)));
        assert_eq!(segment.proposer_vote_of_level(4), Some((proposer_blocks[4], 3)));
        assert_eq!(segment.proposer_vote_of_level(5), Some((proposer_blocks[5], 1)));
        assert_eq!(segment.proposer_vote_of_level(6), Some((proposer_blocks[6], 1)));
        assert_eq!(segment.proposer_vote_of_level(7), None);
    }
}