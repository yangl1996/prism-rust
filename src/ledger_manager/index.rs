use crate::crypto::hash::H256;
use std::sync::{Arc, Weak};
use std::sync::Mutex;
use std::convert::TryFrom;
use crate::chain::*;
use std::collections::{HashMap, HashSet};
use std::iter::{IntoIterator, FromIterator};
use statrs::distribution::{Discrete, Poisson, Univariate};
use log::{info, warn};
use std::ops::Range;
use crate::config::*;

pub struct LedgerIndex {
    voter_tips: Vec<Arc<Voter>>,
    proposer_tip: Arc<Proposer>,
    unconfirmed_proposer: Arc<Mutex<HashSet<H256>>>,
    leader_sequence: Vec<Option<H256>>,
    ledger_order: Vec<Vec<H256>>,
    config: BlockchainConfig,
}

impl LedgerIndex {
    // TODO: for now, we only have the ability to start from scratch
    pub fn new(proposer_tip: &Arc<Proposer>, voter_tips: &[Arc<Voter>], unconfirmed: &Arc<Mutex<HashSet<H256>>>,
                      leader_sequence: &[Option<H256>], ledger_order: &[Vec<H256>], config: &BlockchainConfig) -> Self
              {
                  Self {
                      voter_tips: voter_tips.to_vec(),
                      proposer_tip: Arc::clone(&proposer_tip),
                      unconfirmed_proposer: Arc::clone(&unconfirmed),
                      leader_sequence: leader_sequence.to_vec(),
                      ledger_order: ledger_order.to_vec(),
                      config: config.clone(),
                  }
              }

    // returns added transaction blocks, removed transaction blocks
    pub fn advance_ledger_to(&mut self, new_voter_tips: &[Arc<Voter>], proposer_index: &ChainIndex<Proposer>) -> (Vec<H256>, Vec<H256>) {
        // track the range of the proposer levels that may see a change in the votes
        let mut affected_range: Range<u64> = Range {
            start: std::u64::MAX,
            end: std::u64::MIN,
        };
        let num_voter_chains = self.voter_tips.len();
        if num_voter_chains != new_voter_tips.len() {
            panic!("New voter ledger tips has different number of voter chains from the current tips");
        }
        for chain_num in 0..num_voter_chains {
            let range = new_voter_tips[chain_num].affected_range(&self.voter_tips[chain_num]);
            if affected_range.start > range.0 {
                affected_range.start = range.0;
            }
            if affected_range.end < range.1 {
                affected_range.end = range.1;
            }
            self.voter_tips[chain_num] = Arc::clone(&new_voter_tips[chain_num]);
        }

        // recompute the leader of each level that was affected
        // we will recompute the leader starting from min. affected level or ledger tip level + 1,
        // whichever is smaller. so make min. affected level the smaller of the two
        if affected_range.start < affected_range.end {
            let proposer_ledger_tip = self.proposer_tip.level();
            if proposer_ledger_tip + 1 < affected_range.start {
                affected_range.start = proposer_ledger_tip + 1;
            }
        }

        // start actually recomputing the leaders
        let mut change_begin: Option<u64> = None;   // marks the level where we see the first chain of leader
        for level in affected_range {
            // get the current leader of the level. if the level is empty now, insert a None
            let existing_leader = match self.leader_sequence.get(usize::try_from(level).unwrap()) {
                Some(v) => *v,
                None => {
                    if self.leader_sequence.len() != usize::try_from(level).unwrap() {
                        panic!("The length of the leader sequence vector is not the same as the level of the proposer ledger tip");
                    }
                    self.leader_sequence.push(None);
                    None
                }
            };
            // calculate two different leaders for confirm and deconfirm respectively
            // we set a higher bar to confirm so that we don't deconfirm easily
            let new_leader_confirm: Option<H256> =
                proposer_leader(&new_voter_tips, level, self.config.quantile_epsilon_confirm, self.config.adversary_ratio);
            let new_leader_deconfirm: Option<H256> =
                proposer_leader(&new_voter_tips, level, self.config.quantile_epsilon_deconfirm, self.config.adversary_ratio);
            let new_leader = {
                if existing_leader.is_some() {
                    new_leader_deconfirm
                } else {
                    new_leader_confirm
                }
            };

            if new_leader != existing_leader {
                match new_leader {
                    Some(hash) => info!(
                        "New proposer leader selected for level {}: {:.8}",
                        level, hash
                        ),
                    None => warn!("Proposer leader deconfirmed for level {}", level),
                }
                // mark it's the beginning of the change
                if change_begin.is_none() {
                    change_begin = Some(level);
                }
                // write it to the leader sequence vector
                self.leader_sequence[usize::try_from(level).unwrap()] = new_leader;
            }
        }

        // recompute the ledger from the first level whose leader changed
        if let Some(change_begin) = change_begin {
            let previous_proposer_tip_level = self.proposer_tip.level();
            let mut removed: Vec<H256> = vec![];
            let mut added: Vec<H256> = vec![];            

            // deconfirm the blocks from change_begin all the way to previous ledger tip
            let mut unconfirmed_ptr = self.unconfirmed_proposer.lock().unwrap();
            for level in change_begin..=previous_proposer_tip_level {
                let original_ledger = &self.ledger_order[usize::try_from(level).unwrap()];
                for block in original_ledger.iter() {
                    // the proposer block is now unconfirmed
                    unconfirmed_ptr.insert(*block);
                    removed.push(*block);
                }
                self.ledger_order[usize::try_from(level).unwrap()] = vec![];
            }
            drop(unconfirmed_ptr);

            // recompute the ledger from change_begin until the first level where there's no leader
            // make sure that the ledger is continuous
            if change_begin <= previous_proposer_tip_level + 1 {
                let mut unconfirmed_ptr = self.unconfirmed_proposer.lock().unwrap();
                for level in change_begin.. {
                    let leader = match self.leader_sequence.get(usize::try_from(level).unwrap()) {
                        Some(l) => match l {
                            Some(v) => v,
                            None => {
                                break;
                            }
                        }
                        None => {
                            break;
                        }
                    };

                    // Get the sequence of blocks by doing a depth-first traverse
                    let leader = std::sync::Arc::clone(&proposer_index.blocks.get(&leader).unwrap());
                    self.proposer_tip = std::sync::Arc::clone(&leader);

                    // first deref into HashSet, then take ref to get IntoIter<'a>
                    let seq = leader.ledger_order(&(*unconfirmed_ptr));
                    let order: Vec<H256> = seq.into_iter().map(|x| x.hash()).collect();

                    // deduplicate, keep the one copy that is former in this order
                    let order: Vec<H256> = order
                        .into_iter()
                        .filter(|h| unconfirmed_ptr.remove(h))
                        .collect();
                    added.extend(&order);
                    match self.ledger_order.get_mut(usize::try_from(level).unwrap()) {
                        Some(ptr) => {
                            *ptr = order;
                        }
                        None => {
                            if self.ledger_order.len() != usize::try_from(level).unwrap() {
                                panic!("The length of the ledger order vector is not the same as the level of the proposer ledger");
                            }
                            self.ledger_order.push(order);
                        }
                    } 
                }
                drop(unconfirmed_ptr);
            }

            let mut removed_transaction_blocks: Vec<H256> = vec![];
            let mut added_transaction_blocks: Vec<H256> = vec![];
            for block in &removed {
                let ptr = proposer_index.blocks.get(&block).unwrap();
                removed_transaction_blocks.extend(ptr.transaction_block_refs());
            }
            for block in &added {
                let ptr = proposer_index.blocks.get(&block).unwrap();
                added_transaction_blocks.extend(ptr.transaction_block_refs());
            }
            (added_transaction_blocks, removed_transaction_blocks)
        } else {
            (vec![], vec![])
        }
    }
}

fn proposer_leader(voter_tips: &[Arc<Voter>], level: u64, quantile: f32, adversary_ratio: f32) -> Option<H256> 
{
    // compute the new leader of this level
    // we use the confirmation policy from https://arxiv.org/abs/1810.08092
    let mut new_leader: Option<H256> = None;

    // collect the depth of each vote on each proposer block
    let mut votes_depth: HashMap<H256, Vec<u64>> = HashMap::new(); // chain number and vote depth cast on the proposer block

    // collect the total votes on all proposer blocks of the level, and the number of
    // voter blocks mined on the main chain after those votes are casted
    let mut total_vote_count: u16 = 0;
    let mut total_vote_blocks: u64 = 0;

    // get the vote from each voter chain
    for voter in voter_tips.iter() {
        let vote = voter.proposer_vote_of_level(level);
        // if this chain voted
        if let Some((hash, depth)) = vote {
            if let Some(l) = votes_depth.get_mut(&hash) {
                l.push(depth);
            } else {
                votes_depth.insert(hash, vec![depth]);
            }
            total_vote_count += 1;
            // count the number of blocks on main chain starting at the vote
            total_vote_blocks += depth;
        }
    }
    let proposer_blocks: Vec<H256> = votes_depth.keys().copied().collect();
    let num_voter_chains = u16::try_from(voter_tips.len()).unwrap();

    // no point in going further if less than 3/5 votes are cast
    if total_vote_count > num_voter_chains * 3 / 5 {
        // calculate the average number of voter blocks mined after
        // a vote is casted. we use this as an estimator of honest mining
        // rate, and then derive the believed malicious mining rate
        let avg_vote_blocks = total_vote_blocks as f32 / f32::from(total_vote_count);
        // expected voter depth of an adversary
        let adversary_expected_vote_depth =
            avg_vote_blocks / (1.0 - adversary_ratio) * adversary_ratio;
        let poisson = Poisson::new(f64::from(adversary_expected_vote_depth)).unwrap();

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
                    let p2 = (adversary_ratio
                              / (1.0 - adversary_ratio))
                        .powi((depth - k + 1) as i32);
                    p += p1 * p2;
                }
                block_votes_mean += 1.0 - p;
                block_votes_variance += p * (1.0 - p);
            }
            // using gaussian approximation
            let tmp = block_votes_mean - (block_votes_variance).sqrt() * quantile;
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
            if (max_vote_lcb - block_votes_lcb).abs() < std::f32::EPSILON
                && new_leader.is_some()
                {
                    // TODO: is_some required?
                    if *block < new_leader.unwrap() {
                        new_leader = Some(*block);
                    }
                }
        }
        // check if the lcb_vote of new_leader is bigger than second best ucb votes
        let remaining_votes = f32::from(num_voter_chains) - total_votes_lcb;

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
                        new_leader = None;
                        break;
                    }
                //In case of a tie, choose block with lower hash.
                if (max_vote_lcb - (votes_lcb.get(p_block).unwrap() + remaining_votes)).abs()
                    < std::f32::EPSILON
                        && *p_block < new_leader.unwrap()
                        {
                            new_leader = None;
                            break;
                        }
            }
        }
    }
    new_leader
}
