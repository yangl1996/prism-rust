use std::sync::atomic::{AtomicUsize, Ordering};
use crate::transaction::Transaction;
use crate::block::Block;
use crate::block::Content as BlockContent;
use crate::wallet::WalletError;

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
    processed_proposer_blocks: AtomicUsize,
    processed_proposer_block_bytes: AtomicUsize,
    processed_voter_blocks: AtomicUsize,
    processed_voter_block_bytes: AtomicUsize,
    processed_transaction_blocks: AtomicUsize,
    processed_transaction_block_bytes: AtomicUsize,
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
            generated_transactions: AtomicUsize::new(0),
            generated_transaction_bytes: AtomicUsize::new(0),
            generate_transaction_failures: AtomicUsize::new(0),
            confirmed_transactions: AtomicUsize::new(0),
            confirmed_transaction_bytes: AtomicUsize::new(0),
            deconfirmed_transactions: AtomicUsize::new(0),
            deconfirmed_transaction_bytes: AtomicUsize::new(0),
            processed_proposer_blocks: AtomicUsize::new(0),
            processed_proposer_block_bytes: AtomicUsize::new(0),
            processed_voter_blocks: AtomicUsize::new(0),
            processed_voter_block_bytes: AtomicUsize::new(0),
            processed_transaction_blocks: AtomicUsize::new(0),
            processed_transaction_block_bytes: AtomicUsize::new(0),
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
        return Snapshot {
            generated_transactions: self.generated_transactions.load(Ordering::Relaxed),
            generated_transaction_bytes: self.generated_transaction_bytes.load(Ordering::Relaxed),
            generate_transaction_failures: self.generate_transaction_failures.load(Ordering::Relaxed),
            confirmed_transactions: self.confirmed_transactions.load(Ordering::Relaxed),
            confirmed_transaction_bytes: self.confirmed_transaction_bytes.load(Ordering::Relaxed),
            deconfirmed_transactions: self.deconfirmed_transactions.load(Ordering::Relaxed),
            deconfirmed_transaction_bytes: self.deconfirmed_transaction_bytes.load(Ordering::Relaxed),
            processed_proposer_blocks: self.processed_proposer_blocks.load(Ordering::Relaxed),
            processed_proposer_block_bytes: self.processed_proposer_block_bytes.load(Ordering::Relaxed),
            processed_voter_blocks: self.processed_voter_blocks.load(Ordering::Relaxed),
            processed_voter_block_bytes: self.processed_voter_block_bytes.load(Ordering::Relaxed),
            processed_transaction_blocks: self.processed_transaction_blocks.load(Ordering::Relaxed),
            processed_transaction_block_bytes: self.processed_transaction_block_bytes.load(Ordering::Relaxed),
        };
    }
}
