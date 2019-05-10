use super::*;
use crate::block::Block;
use bigint::uint::U256;
use crate::block::Content;
use std::sync::{Arc, Mutex};
use super::transaction::*;
use crate::utxodb::UtxoDatabase;

fn validate(block: &Block, utxodb: &UtxoDatabase) -> BlockResult {
    let content = match &block.content {
        Content::Transaction(content) => content,
        _ => panic!("Wrong type"),
    };
    for transaction in content.transactions.iter() {
        if !check_non_empty(&transaction) {
            return BlockResult::Fail;
        }
        if !check_input_unspent(&transaction, utxodb) {
            return BlockResult::Fail;
        }
        if !check_sufficient_input(&transaction) {
            return BlockResult::Fail;
        }
        if !check_signature(&transaction) {
            return BlockResult::Fail;
        }
    }
    return BlockResult::Pass;
}
