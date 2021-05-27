use crate::crypto::hash::H256;
use bigint::uint::U256;

const AVG_TX_SIZE: u32 = 168; // average size of a transaction (in Bytes)
const PROPOSER_TX_REF_HEADROOM: f32 = 10.0;
const SORTITION_PRECISION: u64 = std::u64::MAX;

// Chain IDs
pub const PROPOSER_INDEX: u16 = 0;
pub const TRANSACTION_INDEX: u16 = 1;
pub const FIRST_VOTER_INDEX: u16 = 2;

#[derive(Clone)]
pub struct BlockchainConfig {
    /// Number of voter chains.
    pub voter_chains: u16,
    /// Maximum size of a transaction block in terms of transactions.
    pub tx_txs: u32,
    /// Maximum number of transaction block references in a proposer block.
    pub proposer_tx_refs: u32,
    /// Proposer block minng rate in blocks/sec.
    pub proposer_mining_rate: f32,
    /// Voter block minng rate for one voter chain, in blocks/sec.
    pub voter_mining_rate: f32,
    /// Transaction block minng rate in blocks/sec.
    pub tx_mining_rate: f32,
    /// Hash of proposer genesis block.
    pub proposer_genesis: H256,
    /// Hashes of voter genesis blocks.
    pub voter_genesis: Vec<H256>,
    total_mining_rate: f32,
    total_sortition_width: U256,
    proposer_sortition_width: U256,
    voter_sortition_width: U256,
    tx_sortition_width: U256,
    pub adversary_ratio: f32,
    log_epsilon: f32,   // confirmation confidence is 1-exp(-log_epsilon)
    pub network_delta: f32, // network delay
    pub beta: f32,  // adversary_ratio, used in confirmation
    pub small_delta: f32,   // slack in the new confirmation algorithm
    pub use_theory_paper_rule: bool,
}

impl BlockchainConfig {
    pub fn new(
        voter_chains: u16,
        tx_size: u32,
        tx_throughput: u32,
        proposer_rate: f32,
        voter_rate: f32,
        adv_ratio: f32,
        log_epsilon: f32,
        network_delta: f32,
        beta: f32,
        avg_latency: f32,
        use_theory_paper_rule: bool,
    ) -> Self {
        let tx_txs = tx_size / AVG_TX_SIZE;
        let proposer_genesis: H256 = {
            let mut raw_hash: [u8; 32] = [0; 32];
            let bytes = PROPOSER_INDEX.to_be_bytes();
            raw_hash[30] = bytes[0];
            raw_hash[31] = bytes[1];
            raw_hash.into()
        };
        let voter_genesis_hashes: Vec<H256> = {
            let mut v: Vec<H256> = vec![];
            for chain_num in 0..voter_chains {
                let mut raw_hash: [u8; 32] = [0; 32];
                let bytes = (chain_num + FIRST_VOTER_INDEX).to_be_bytes();
                raw_hash[30] = bytes[0];
                raw_hash[31] = bytes[1];
                v.push(raw_hash.into());
            }
            v
        };
        let tx_mining_rate: f32 = {
            let tx_thruput: f32 = tx_throughput as f32;
            let tx_txs: f32 = tx_txs as f32;
            tx_thruput / tx_txs
        };
        let total_mining_rate: f32 =
            proposer_rate + voter_rate * f32::from(voter_chains) + tx_mining_rate;
        let proposer_width: u64 = {
            let precise: f32 = (proposer_rate / total_mining_rate) * SORTITION_PRECISION as f32;
            precise.ceil() as u64
        };
        let voter_width: u64 = {
            let precise: f32 = (voter_rate / total_mining_rate) * SORTITION_PRECISION as f32;
            precise.ceil() as u64
        };
        let tx_width: u64 =
            SORTITION_PRECISION - proposer_width - voter_width * u64::from(voter_chains);
        Self {
            voter_chains,
            tx_txs,
            proposer_tx_refs: (tx_mining_rate / proposer_rate * PROPOSER_TX_REF_HEADROOM).ceil()
                as u32,
            proposer_mining_rate: proposer_rate,
            voter_mining_rate: voter_rate,
            tx_mining_rate,
            proposer_genesis,
            voter_genesis: voter_genesis_hashes,
            total_mining_rate,
            total_sortition_width: SORTITION_PRECISION.into(),
            proposer_sortition_width: proposer_width.into(),
            voter_sortition_width: voter_width.into(),
            tx_sortition_width: tx_width.into(),
            adversary_ratio: adv_ratio,
            log_epsilon,
            network_delta,
            beta,
            small_delta: solve_small_delta(voter_rate * (1.0 - beta), voter_rate * beta, network_delta, (-log_epsilon).exp(), 23800, avg_latency),
            use_theory_paper_rule,
        }
    }

    pub fn sortition_hash(&self, hash: &H256, difficulty: &H256) -> Option<u16> {
        let hash = U256::from_big_endian(hash.as_ref());
        let difficulty = U256::from_big_endian(difficulty.as_ref());
        let multiplier = difficulty / self.total_sortition_width;

        let proposer_width = multiplier * self.proposer_sortition_width;
        let transaction_width =
            multiplier * (self.proposer_sortition_width + self.tx_sortition_width);
        if hash < proposer_width {
            Some(PROPOSER_INDEX)
        } else if hash < (transaction_width + proposer_width) {
            Some(TRANSACTION_INDEX)
        } else if hash < difficulty {
            let voter_idx = (hash - proposer_width - transaction_width) % self.voter_chains.into();
            Some(voter_idx.as_u32() as u16 + FIRST_VOTER_INDEX)
        } else {
            None
        }
    }

    pub fn try_confirm(&self, depth_sum: u64, nonvote: u64) -> bool {
        let lh_p = lh_prime(self.voter_mining_rate * (1.0 - self.beta), self.network_delta);
        let t = depth_sum as f32 / ((1.0 + self.small_delta) * self.voter_chains as f32 * self.voter_mining_rate);
        let th = nonvote as f32 / self.voter_chains as f32 + 0.5 + self.small_delta;
        let h_delta = 1.0 - function_q(t, lh_p, self.voter_mining_rate * self.beta);
        if h_delta >= th {
            return true;
        } else {
            //println!("h_delta={}, threshold={}", h_delta, th);
            return false;
        }
    }

    fn delta_d(&self, d: u64) -> f32 {
        // f_v^bar: prob that a voter block is mined in a round
        let term_1 = 1.0 / (4.0 * self.voter_mining_rate * (d as f32));
        let term_2 = (1.0 - 2.0 * self.beta) / (8.0 * (self.voter_chains as f32).ln());
        if term_1 > term_2 {
            term_1
        }
        else {
            term_2
        }
    }

    fn v_n_lowerbound(&self, v_d: &[u64]) -> f32 {
        // v_d: the i-th item is the number of votes that is at least i+1 deep
        let mut max: f32 = 0.0;
        for (pos, n) in v_d.iter().enumerate() {
            let depth = pos + 1;
            let val = *n as f32 - 2.0 * self.delta_d(depth as u64) * (self.voter_chains as f32);
            if val > max {
                max = val;
            }
        }
        return max;
    }
    
    pub fn try_confirm_theory_paper(&self, vote_depth: &[u64]) -> bool {
        // see the theory paper
        // we assume the round interval is 1 sec
        // this function should only be called on the proposer block with the most votes
        let mut count: Vec<u64> = vec![];
        let mut threshold = 1;
        loop {
            let mut num = 0;
            for i in vote_depth.iter() {
                if *i >= threshold {
                    num += 1;
                }
            }
            count.push(num);
            if num == 0 {
                break;
            }
            threshold += 1;
        }

        
        let v_lowerbound = self.v_n_lowerbound(&count);
        // simply ignore the other public proposer blocks and their votes
        // because we are not doing list confirmation
        let v_private_upperbound = self.voter_chains as f32 - v_lowerbound;
        if v_lowerbound > v_private_upperbound {
            true
        }
        else {
            false
        }
    }

    // TODO: just make a table of the inverse of function_q for different Vl(T) values
}

pub fn function_q(t_l: f32, lh_l: f32, la_l: f32) -> f32 {
    let t = t_l as f64;
    let lh = lh_l as f64;
    let la = la_l as f64;

    let terms = 40;

    let mut res: f64 = 1.0;
    for l in 0..=terms {
        let term1: f64 = (lh - la) / lh * (la / lh).powi(l as i32);
        let mut term2: f64 = 0.0;
        for k in l..=terms {
            let term2_1: f64 = (-lh * t).exp() * factdiv(lh * t, k);
            let mut term2_2: f64 = 0.0;
            for n in 0..=k-l {
                let term2_2_1: f64 = (-la * t).exp() * factdiv(la * t, n) * (1.0 - (la / lh).powi((k-n-l) as i32));
                term2_2 = term2_2 + term2_2_1;
            }
            term2 = term2 + term2_1 * term2_2;
        }
        res = res - term1 * term2;
    }
    return res as f32;
}

fn lh_prime(lh: f32, delta: f32) -> f32 {
    return lh / (1.0 + lh * delta);
}

fn solve_t_delta(lh: f32, la: f32, delta: f32, small_delta: f32, avg_latency: f32) -> f32 {
    let mut res: f32 = 1.0;
    let lh_p = lh_prime(lh, delta);
    loop {
        let h_delta: f32 = 1.0 - function_q(res, lh_p, la);
        let th = 0.5 + small_delta;
        let d: f32 = if h_delta > th {
            h_delta - th
        } else {
            th - h_delta
        };
        if d < 0.01 {
            let fork = 1.0 - 1.0 / (1.0 + (lh + la) * avg_latency);
            let explatency = res / (1.0 - fork);
            println!("Expected confirmation latency={}", explatency);
            return res
        } else {
            res += 1.0;
            //println!("trying t_delta={}, diff={}", res, d);
        }
    }
}

fn error_prob(l: f32, m: u16, small_delta: f32, t_delta: f32) -> f32 {
    return (-2.0 * small_delta * small_delta * m as f32).exp() + 2.0 * (-(small_delta * small_delta * t_delta * l * m as f32) / 3.0).exp();
}

fn solve_small_delta(lh: f32, la: f32, delta: f32, ep: f32, m: u16, avg_latency: f32) -> f32 {
    let mut res: f32 = 0.0;
    for _ in 0..5 {
        let mut tmp_res: f32 = 0.001;
        let t_delta = solve_t_delta(lh, la, delta, res, avg_latency);
        loop {
            let our_ep = error_prob(lh + la, m, tmp_res, t_delta);
            if our_ep > ep {
                tmp_res += 0.001;
            } else {
                println!("Small delta={}, error prob={} < {}", tmp_res, our_ep, ep);
                res = tmp_res;
                break;
            }
        }
    }
    return res;
}

fn factdiv(up: f64, n: u64) -> f64 {
    let mut res: f64 = 1.0;
    for i in 1..=n {
        res = res * up / i as f64;
    }
    return res;
}

lazy_static! {
    pub static ref DEFAULT_DIFFICULTY: H256 = {
        let raw: [u8; 32] = [255; 32];
        raw.into()
    };
}
