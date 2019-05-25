use std::sync::atomic::{AtomicUsize, Ordering};
use crate::transaction::Transaction;

pub trait PayloadSize {
    fn size(&self) -> usize;
}

pub struct Counter {
    confirmed_transactions: AtomicUsize,
    confirmed_transaction_bytes: AtomicUsize,
    processed_proposer_blocks: AtomicUsize,
    processed_proposer_block_bytes: AtomicUsize,
    processed_voter_blocks: AtomicUsize,
    processed_voter_block_bytes: AtomicUsize,
    processed_transaction_blocks: AtomicUsize,
    processed_transaction_block_bytes: AtomicUsize,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub confirmed_transactions: usize,
    pub confirmed_transaction_bytes: usize,
    pub processed_proposer_blocks: usize,
    pub processed_proposer_block_bytes: usize,
    pub processed_voter_blocks: usize,
    pub processed_voter_block_bytes: usize,
    pub processed_transaction_blocks: usize,
    pub processed_transaction_block_bytes: usize,
}

impl Counter {
    pub fn new() -> Self {
        return Self {
            confirmed_transactions: AtomicUsize::new(0),
            confirmed_transaction_bytes: AtomicUsize::new(0),
            processed_proposer_blocks: AtomicUsize::new(0),
            processed_proposer_block_bytes: AtomicUsize::new(0),
            processed_voter_blocks: AtomicUsize::new(0),
            processed_voter_block_bytes: AtomicUsize::new(0),
            processed_transaction_blocks: AtomicUsize::new(0),
            processed_transaction_block_bytes: AtomicUsize::new(0),
        }
    }

    pub fn record_confirmed_transaction(&self, t: &Transaction) {
        self.confirmed_transactions.fetch_add(1, Ordering::Relaxed);
        self.confirmed_transaction_bytes.fetch_add(t.size(), Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> Snapshot {
        return Snapshot {
            confirmed_transactions: self.confirmed_transactions.load(Ordering::Relaxed),
            confirmed_transaction_bytes: self.confirmed_transaction_bytes.load(Ordering::Relaxed),
            processed_proposer_blocks: self.processed_proposer_blocks.load(Ordering::Relaxed),
            processed_proposer_block_bytes: self.processed_proposer_block_bytes.load(Ordering::Relaxed),
            processed_voter_blocks: self.processed_voter_blocks.load(Ordering::Relaxed),
            processed_voter_block_bytes: self.processed_voter_block_bytes.load(Ordering::Relaxed),
            processed_transaction_blocks: self.processed_transaction_blocks.load(Ordering::Relaxed),
            processed_transaction_block_bytes: self.processed_transaction_block_bytes.load(Ordering::Relaxed),
        };
    }
}
