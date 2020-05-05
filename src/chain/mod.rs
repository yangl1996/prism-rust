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
use crate::crypto::hash::{H256, Hashable};
use std::sync::Arc;
use std::convert::TryFrom;

pub struct Segment {
    pub lowest_vote_level: u64,     // inclusive
    pub highest_vote_level: u64,    // inclusive
    pub lowest_block: (H256, u64),
    pub highest_block: (H256, u64),
    pub parent: Option<Arc<Segment>>,
    pub votes: Vec<(H256, u64)>,    // proposer hash, level of the vote
}

/// Given the voter chain (identified by its deepest segment) and the proposer level we are
/// interested in, get the proposer block on that level voted by this voter chain and the depth of
/// the vote.
pub fn proposer_vote_of_level(voter_chain: &Segment, proposer_level: u64) -> Option<(H256, u64)> {
    // For now, we simply do a linear search. TODO: implement a functional segment tree to improve
    // the speed that we search the segment containing a specific proposer level.

    let best_voter_level = voter_chain.highest_block.1;
    // First check the current segment
    if proposer_level > voter_chain.highest_vote_level {
        return None;
    }
    else {
        if proposer_level >= voter_chain.lowest_vote_level {
            let idx = usize::try_from(proposer_level - voter_chain.lowest_vote_level).unwrap();
            let (vote, level) = voter_chain.votes[idx];
            return Some((vote, best_voter_level - level + 1));
        }
    }

    // Then trace back and find the first segment that votes a lower level than proposer_level
    let mut current_segment = match &voter_chain.parent {
        Some(p) => Arc::clone(&p),
        None => return None,
    };
    while proposer_level < current_segment.lowest_vote_level {
        current_segment = match &current_segment.parent {
            Some(p) => Arc::clone(&p),
            None => return None,
        };
    }
    if proposer_level <= current_segment.highest_vote_level {
            let idx = usize::try_from(proposer_level - current_segment.lowest_vote_level).unwrap();
            let (vote, level) = current_segment.votes[idx];
            return Some((vote, best_voter_level - level + 1));
    }
    else {
        None
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

    #[test]
    fn get_proposer_vote_by_level() {
        // generate 7 hashes for proposer blocks, and 6 hashes for voter blocks
        let mut proposer_blocks: Vec<H256> = vec![];
        for _ in 0..7 {
            proposer_blocks.push(get_hash());
        }
        let mut voter_blocks: Vec<H256> = vec![];
        for _ in 0..6 {
            voter_blocks.push(get_hash());
        }

        let genesis_segment = Segment {
            lowest_vote_level: 0,
            highest_vote_level: 3,
            lowest_block: (voter_blocks[0], 0),
            highest_block: (voter_blocks[2], 2),
            parent: None,
            votes: vec![(proposer_blocks[0], 0), (proposer_blocks[1], 1), (proposer_blocks[2], 1), (proposer_blocks[3], 2)],
        };
        let segment_0 = Arc::new(genesis_segment);
        let segment = Segment {
            lowest_vote_level: 4,
            highest_vote_level: 4,
            lowest_block: (voter_blocks[3], 3),
            highest_block: (voter_blocks[4], 4),
            parent: Some(segment_0),
            votes: vec![(proposer_blocks[4], 4)],
        };
        let segment_1 = Arc::new(segment);
        let segment = Segment {
            lowest_vote_level: 5,
            highest_vote_level: 6,
            lowest_block: (voter_blocks[3], 5),
            highest_block: (voter_blocks[3], 5),
            parent: Some(segment_1),
            votes: vec![(proposer_blocks[5], 5), (proposer_blocks[6], 5)],
        };
        assert_eq!(proposer_vote_of_level(&segment, 0), Some((proposer_blocks[0], 6)));
        assert_eq!(proposer_vote_of_level(&segment, 1), Some((proposer_blocks[1], 5)));
        assert_eq!(proposer_vote_of_level(&segment, 2), Some((proposer_blocks[2], 5)));
        assert_eq!(proposer_vote_of_level(&segment, 3), Some((proposer_blocks[3], 4)));
        assert_eq!(proposer_vote_of_level(&segment, 4), Some((proposer_blocks[4], 2)));
        assert_eq!(proposer_vote_of_level(&segment, 5), Some((proposer_blocks[5], 1)));
        assert_eq!(proposer_vote_of_level(&segment, 6), Some((proposer_blocks[6], 1)));
        assert_eq!(proposer_vote_of_level(&segment, 7), None);
    }
}
