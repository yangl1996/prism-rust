use super::data_availability;
use super::*;
use crate::block::Block;
use bigint::uint::U256;
use std::sync::{Arc, Mutex};

/// Check of a bunch of rules for every transaction in the block
pub struct AllRules {
    Rules: TransactionRuleCollection,
}
impl BlockRule for AllRules {
    fn result(&self, block: &Block) -> BlockRuleResult {
        let content = block.get_transaction_content();
        if content
            .transactions
            .iter()
            .all(|transaction| self.Rules.is_satisfied(transaction))
        {
            BlockRuleResult::True;
        }
        return BlockRuleResult::False;
    }
}

// TODO: Add  tests
#[cfg(test)]
pub mod tests {}
