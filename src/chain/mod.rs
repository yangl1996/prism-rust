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
use crate::block::Block;
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

    pub fn attach_new_block(self: &Arc<Self>, hash: H256, votes: &[H256]) -> Self {
        Self {
            level: self.level + 1,
            hash,
            vote_start_level: self.vote_start_level + u64::try_from(self.votes.len()).unwrap(),
            votes: votes.to_vec(),
            parent: Arc::downgrade(self),
        }
    }
}

//        -----      ------
//       /          /
// -------------------------------
//                \
//                 --------
// For now, we set a threshold level and remove everything lower than that level
pub struct VoterIndex {
    blocks: std::collections::HashMap<H256, Arc<Voter>>,
    starting_level: u64,                                    // the level stored by index 0 in the vecdeque
    by_level: std::collections::VecDeque<Vec<H256>>,        // organized by level in increasing order
}

impl VoterIndex {
    pub fn new(starting: u64) -> Self {
        Self {
            blocks: std::collections::HashMap::new(),
            starting_level: starting,
            by_level: std::collections::VecDeque::new(),
        }
    }

    /*
    pub fn insert_at(&mut self, block: &Block, level: u64) -> Option<Voter> {

    }
    */

    pub fn insert(&mut self, block: &Block, hash: H256) -> Arc<Voter> {
        let content = match &block.content {
            Content::Voter(stuff) => stuff,
            _ => panic!("Adding a non-voter block to a voter chain"),
        };
        let parent = content.voter_parent;
        let parent_ref = match self.blocks.get(&parent) {
            Some(v) => v,
            None => panic!("Adding a voter block whose parent is unknown"),
        };
        
        let new_block = parent_ref.attach_new_block(hash, &content.votes);
        let level = new_block.level;
        let new_block_ref = Arc::new(new_block);
        self.blocks.insert(hash, Arc::clone(&new_block_ref));
        let level_idx = level - self.starting_level;
        let vec_len = u64::try_from(self.by_level.len()).unwrap();
        if level_idx > vec_len {
            panic!("Adding a voter block deeper than the current deepest level + 1");
        }
        else if level_idx == vec_len {
            self.by_level.push_back(vec![hash]);
        }
        else {
            let list = self.by_level.get_mut(usize::try_from(level_idx).unwrap()).unwrap();
            if list.contains(&hash) {
                panic!("Adding a voter block already there on that level");
            }
            list.push(hash);
        }

        return new_block_ref;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

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
        let v = my_voters[0].attach_new_block(voter_blocks[1], &vec![]);
        my_voters.push(Arc::new(v));
        let v = my_voters[1].attach_new_block(voter_blocks[2], &vec![proposer_blocks[1], proposer_blocks[2]]);
        my_voters.push(Arc::new(v));
        let v = my_voters[2].attach_new_block(voter_blocks[3], &vec![proposer_blocks[3]]);
        my_voters.push(Arc::new(v));
        let v = my_voters[3].attach_new_block(voter_blocks[4], &vec![proposer_blocks[4]]);
        my_voters.push(Arc::new(v));
        let v = my_voters[4].attach_new_block(voter_blocks[5], &vec![]);
        my_voters.push(Arc::new(v));
        let v = my_voters[5].attach_new_block(voter_blocks[6], &vec![proposer_blocks[5], proposer_blocks[6]]);
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
