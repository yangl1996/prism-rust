use super::*;
use crate::block::Block;
use bigint::uint::U256;
use std::sync::{Arc, Mutex};

/// Check of a bunch of rules for every transaction in the block
pub struct TransactionBlockRule {
    Rules: TransactionRuleCollection,
}
impl BlockRule for TransactionBlockRule {
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