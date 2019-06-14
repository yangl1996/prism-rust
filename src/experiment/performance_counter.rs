use std::sync::atomic::{AtomicUsize, AtomicIsize, Ordering};
use crate::transaction::Transaction;
use crate::block::Block;
use crate::block::Content as BlockContent;
use crate::wallet::WalletError;
use std::time::SystemTime;

lazy_static! {
    pub static ref PERFORMANCE_COUNTER: Counter = {
        Counter::new()
    };
}

pub trait PayloadSize {
    fn size(&self) -> usize;
}

pub struct Counter {
    generated_transactions: AtomicUsize,
    generated_transaction_bytes: AtomicUsize,
    generate_transaction_failures: AtomicUsize,
    confirmed_transactions: AtomicUsize,
    confirmed_transaction_bytes: AtomicUsize,
    deconfirmed_transactions: AtomicUsize,
    deconfirmed_transaction_bytes: AtomicUsize,
    confirmed_transaction_blocks: AtomicUsize,
    deconfirmed_transaction_blocks: AtomicUsize,
    processed_proposer_blocks: AtomicUsize,
    processed_proposer_block_bytes: AtomicUsize,
    processed_voter_blocks: AtomicUsize,
    processed_voter_block_bytes: AtomicUsize,
    processed_transaction_blocks: AtomicUsize,
    processed_transaction_block_bytes: AtomicUsize,
    mined_proposer_blocks: AtomicUsize,
    mined_proposer_block_bytes: AtomicUsize,
    mined_voter_blocks: AtomicUsize,
    mined_voter_block_bytes: AtomicUsize,
    mined_transaction_blocks: AtomicUsize,
    mined_transaction_block_bytes: AtomicUsize,
    total_proposer_block_delay: AtomicUsize,
    total_voter_block_delay: AtomicUsize,
    total_transaction_block_delay: AtomicUsize,
    total_proposer_block_squared_delay: AtomicUsize,
    total_voter_block_squared_delay: AtomicUsize,
    total_transaction_block_squared_delay: AtomicUsize,
    received_proposer_blocks: AtomicUsize,
    received_voter_blocks: AtomicUsize,
    received_transaction_blocks: AtomicUsize,
    incoming_message_queue: AtomicIsize,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub generated_transactions: usize,
    pub generated_transaction_bytes: usize,
    pub generate_transaction_failures: usize,
    pub confirmed_transactions: usize,
    pub confirmed_transaction_bytes: usize,
    pub deconfirmed_transactions: usize,
    pub deconfirmed_transaction_bytes: usize,
    pub confirmed_transaction_blocks: usize,
    pub deconfirmed_transaction_blocks: usize,
    pub processed_proposer_blocks: usize,
    pub processed_proposer_block_bytes: usize,
    pub processed_voter_blocks: usize,
    pub processed_voter_block_bytes: usize,
    pub processed_transaction_blocks: usize,
    pub processed_transaction_block_bytes: usize,
    pub mined_proposer_blocks: usize,
    pub mined_proposer_block_bytes: usize,
    pub mined_voter_blocks: usize,
    pub mined_voter_block_bytes: usize,
    pub mined_transaction_blocks: usize,
    pub mined_transaction_block_bytes: usize,
    pub proposer_block_delay_mean: usize,
    pub voter_block_delay_mean: usize,
    pub transaction_block_delay_mean: usize,
    pub proposer_block_delay_variance: usize,
    pub voter_block_delay_variance: usize,
    pub transaction_block_delay_variance: usize,
    pub incoming_message_queue: isize,
}

impl Counter {
    pub fn new() -> Self {
        return Self {
            generated_transactions: AtomicUsize::new(0),
            generated_transaction_bytes: AtomicUsize::new(0),
            generate_transaction_failures: AtomicUsize::new(0),
            confirmed_transactions: AtomicUsize::new(0),
            confirmed_transaction_bytes: AtomicUsize::new(0),
            deconfirmed_transactions: AtomicUsize::new(0),
            deconfirmed_transaction_bytes: AtomicUsize::new(0),
            confirmed_transaction_blocks: AtomicUsize::new(0),
            deconfirmed_transaction_blocks: AtomicUsize::new(0),
            processed_proposer_blocks: AtomicUsize::new(0),
            processed_proposer_block_bytes: AtomicUsize::new(0),
            processed_voter_blocks: AtomicUsize::new(0),
            processed_voter_block_bytes: AtomicUsize::new(0),
            processed_transaction_blocks: AtomicUsize::new(0),
            processed_transaction_block_bytes: AtomicUsize::new(0),
            mined_proposer_blocks: AtomicUsize::new(0),
            mined_proposer_block_bytes: AtomicUsize::new(0),
            mined_voter_blocks: AtomicUsize::new(0),
            mined_voter_block_bytes: AtomicUsize::new(0),
            mined_transaction_blocks: AtomicUsize::new(0),
            mined_transaction_block_bytes: AtomicUsize::new(0),
            total_proposer_block_delay: AtomicUsize::new(0),
            total_voter_block_delay: AtomicUsize::new(0),
            total_transaction_block_delay: AtomicUsize::new(0),
            total_proposer_block_squared_delay: AtomicUsize::new(0),
            total_voter_block_squared_delay: AtomicUsize::new(0),
            total_transaction_block_squared_delay: AtomicUsize::new(0),
            received_proposer_blocks: AtomicUsize::new(0),
            received_voter_blocks: AtomicUsize::new(0),
            received_transaction_blocks: AtomicUsize::new(0),
            incoming_message_queue: AtomicIsize::new(0),
        }
    }

    pub fn record_process_message(&self) {
        self.incoming_message_queue.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_receive_message(&self) {
        self.incoming_message_queue.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_receive_block(&self, b: &Block) {
        let mined_time = b.header.timestamp;
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
        let delay = if current_time <= mined_time {
            0
        } else {
            current_time - mined_time
        };
        match b.content {
            BlockContent::Transaction(_) => {
                self.total_transaction_block_delay.fetch_add(delay as usize, Ordering::Relaxed);
                self.total_transaction_block_squared_delay.fetch_add((delay * delay) as usize, Ordering::Relaxed);
                self.received_transaction_blocks.fetch_add(1, Ordering::Relaxed);
            }
            BlockContent::Proposer(_) => {
                self.total_proposer_block_delay.fetch_add(delay as usize, Ordering::Relaxed);
                self.total_proposer_block_squared_delay.fetch_add((delay * delay) as usize, Ordering::Relaxed);
                self.received_proposer_blocks.fetch_add(1, Ordering::Relaxed);
            }
            BlockContent::Voter(_) => {
                self.total_voter_block_delay.fetch_add(delay as usize, Ordering::Relaxed);
                self.total_voter_block_squared_delay.fetch_add((delay * delay) as usize, Ordering::Relaxed);
                self.received_voter_blocks.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn record_process_block(&self, b: &Block) {
        match b.content {
            BlockContent::Transaction(_) => {
                self.processed_transaction_blocks.fetch_add(1, Ordering::Relaxed);
                self.processed_transaction_block_bytes.fetch_add(b.size(), Ordering::Relaxed);
            }
            BlockContent::Voter(_) => {
                self.processed_voter_blocks.fetch_add(1, Ordering::Relaxed);
                self.processed_voter_block_bytes.fetch_add(b.size(), Ordering::Relaxed);
            }
            BlockContent::Proposer(_) => {
                self.processed_proposer_blocks.fetch_add(1, Ordering::Relaxed);
                self.processed_proposer_block_bytes.fetch_add(b.size(), Ordering::Relaxed);
            }
        }
    }

    pub fn record_mine_block(&self, b: &Block) {
        match b.content {
            BlockContent::Transaction(_) => {
                self.mined_transaction_blocks.fetch_add(1, Ordering::Relaxed);
                self.mined_transaction_block_bytes.fetch_add(b.size(), Ordering::Relaxed);
            }
            BlockContent::Voter(_) => {
                self.mined_voter_blocks.fetch_add(1, Ordering::Relaxed);
                self.mined_voter_block_bytes.fetch_add(b.size(), Ordering::Relaxed);
            }
            BlockContent::Proposer(_) => {
                self.mined_proposer_blocks.fetch_add(1, Ordering::Relaxed);
                self.mined_proposer_block_bytes.fetch_add(b.size(), Ordering::Relaxed);
            }
        }
    }

    pub fn record_confirm_transaction_blocks(&self, num_blocks: usize) {
        self.confirmed_transaction_blocks.fetch_add(num_blocks, Ordering::Relaxed);
    }

    pub fn record_deconfirm_transaction_blocks(&self, num_blocks: usize) {
        self.deconfirmed_transaction_blocks.fetch_add(num_blocks, Ordering::Relaxed);
    }

    pub fn record_confirm_transaction(&self, t: &Transaction) {
        self.confirmed_transactions.fetch_add(1, Ordering::Relaxed);
        self.confirmed_transaction_bytes.fetch_add(t.size(), Ordering::Relaxed);
    }

    pub fn record_deconfirm_transaction(&self, t: &Transaction) {
        self.deconfirmed_transactions.fetch_add(1, Ordering::Relaxed);
        self.deconfirmed_transaction_bytes.fetch_add(t.size(), Ordering::Relaxed);
    }

    pub fn record_generate_transaction(&self, t: &Result<Transaction, WalletError>) {
        match t {
            Ok(t) => {
                self.generated_transactions.fetch_add(1, Ordering::Relaxed);
                self.generated_transaction_bytes.fetch_add(t.size(), Ordering::Relaxed);
            }
            Err(_) => {
                self.generate_transaction_failures.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let proposer_delay_total = self.total_proposer_block_delay.load(Ordering::Relaxed);
        let proposer_delay_squared_total = self.total_proposer_block_squared_delay.load(Ordering::Relaxed);
        let proposer_num = self.received_proposer_blocks.load(Ordering::Relaxed);
        let proposer_delay_mean = if proposer_num == 0 {
            0
        } else {
            proposer_delay_total / proposer_num
        };
        let proposer_delay_variance = if proposer_num == 0 {
            0
        } else {
            proposer_delay_squared_total / proposer_num - (proposer_delay_total / proposer_num) * (proposer_delay_total / proposer_num)
        };
        let voter_delay_total = self.total_voter_block_delay.load(Ordering::Relaxed);
        let voter_delay_squared_total = self.total_voter_block_squared_delay.load(Ordering::Relaxed);
        let voter_num = self.received_voter_blocks.load(Ordering::Relaxed);
        let voter_delay_mean = if voter_num == 0 {
            0
        } else {
            voter_delay_total / voter_num
        };
        let voter_delay_variance = if voter_num == 0 {
            0
        } else {
            voter_delay_squared_total / voter_num - (voter_delay_total / voter_num) * (voter_delay_total / voter_num)
        };
        let transaction_delay_total = self.total_transaction_block_delay.load(Ordering::Relaxed);
        let transaction_delay_squared_total = self.total_transaction_block_squared_delay.load(Ordering::Relaxed);
        let transaction_num = self.received_transaction_blocks.load(Ordering::Relaxed);
        let transaction_delay_mean = if transaction_num == 0 {
            0
        } else {
            transaction_delay_total / transaction_num
        };
        let transaction_delay_variance = if transaction_num == 0 {
            0
        } else {
            transaction_delay_squared_total / transaction_num - (transaction_delay_total / transaction_num) * (transaction_delay_total / transaction_num)
        };
        let incoming_message_queue = self.incoming_message_queue.load(Ordering::Relaxed);
        let incoming_message_queue = if incoming_message_queue < 0 {
            0
        } else {
            incoming_message_queue
        };
        return Snapshot {
            generated_transactions: self.generated_transactions.load(Ordering::Relaxed),
            generated_transaction_bytes: self.generated_transaction_bytes.load(Ordering::Relaxed),
            generate_transaction_failures: self.generate_transaction_failures.load(Ordering::Relaxed),
            confirmed_transactions: self.confirmed_transactions.load(Ordering::Relaxed),
            confirmed_transaction_bytes: self.confirmed_transaction_bytes.load(Ordering::Relaxed),
            deconfirmed_transactions: self.deconfirmed_transactions.load(Ordering::Relaxed),
            deconfirmed_transaction_bytes: self.deconfirmed_transaction_bytes.load(Ordering::Relaxed),
            confirmed_transaction_blocks: self.confirmed_transaction_blocks.load(Ordering::Relaxed),
            deconfirmed_transaction_blocks: self.deconfirmed_transaction_blocks.load(Ordering::Relaxed),
            processed_proposer_blocks: self.processed_proposer_blocks.load(Ordering::Relaxed),
            processed_proposer_block_bytes: self.processed_proposer_block_bytes.load(Ordering::Relaxed),
            processed_voter_blocks: self.processed_voter_blocks.load(Ordering::Relaxed),
            processed_voter_block_bytes: self.processed_voter_block_bytes.load(Ordering::Relaxed),
            processed_transaction_blocks: self.processed_transaction_blocks.load(Ordering::Relaxed),
            processed_transaction_block_bytes: self.processed_transaction_block_bytes.load(Ordering::Relaxed),
            mined_proposer_blocks: self.mined_proposer_blocks.load(Ordering::Relaxed),
            mined_proposer_block_bytes: self.mined_proposer_block_bytes.load(Ordering::Relaxed),
            mined_voter_blocks: self.mined_voter_blocks.load(Ordering::Relaxed),
            mined_voter_block_bytes: self.mined_voter_block_bytes.load(Ordering::Relaxed),
            mined_transaction_blocks: self.mined_transaction_blocks.load(Ordering::Relaxed),
            mined_transaction_block_bytes: self.mined_transaction_block_bytes.load(Ordering::Relaxed),
            proposer_block_delay_mean: proposer_delay_mean,
            proposer_block_delay_variance: proposer_delay_variance,
            voter_block_delay_mean: voter_delay_mean,
            voter_block_delay_variance: voter_delay_variance,
            transaction_block_delay_mean: transaction_delay_mean,
            transaction_block_delay_variance: transaction_delay_variance,
            incoming_message_queue: incoming_message_queue,
        };
    }
}
