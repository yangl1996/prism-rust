use std::sync::{atomic::AtomicUsize, Arc, Mutex};

pub trait PayloadSize {
    fn size(&self) -> usize;
}

pub struct PerformanceCounter {
    confirmed_transactions: AtomicUsize,
    confirmed_transaction_bytes: AtomicUsize,
}
