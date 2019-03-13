use crate::transaction::Transaction;
use crate::miner::memory_pool::MemoryPool;

/// Transaction fee calculator for memory pool
pub trait MemoryPoolFeeCalculator {
    /// Compute transaction fee
    fn calculate(&self, memory_pool: &MemoryPool, tx: &Transaction) -> u64;
}

pub struct FeeIsOne;

impl MemoryPoolFeeCalculator for FeeIsOne {
    fn calculate(&self, memory_pool: &MemoryPool, tx: &Transaction) -> u64 {
        1
    }
}

pub struct FeeIsZero;

impl MemoryPoolFeeCalculator for FeeIsZero {
    fn calculate(&self, memory_pool: &MemoryPool, tx: &Transaction) -> u64 {
        0
    }
}