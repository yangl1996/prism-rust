pub mod data_availability;
pub mod header;
pub mod transaction;
pub mod transaction_block;
pub mod voter_block;
use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::Transaction;
use std::sync::{Arc, Mutex};

/// The common trait for transaction rules
pub trait TransactionRule {
    fn is_satisfied(&self, transaction: &Transaction) -> bool;
}

/// Collection of transaction rules
pub struct TransactionRuleCollection {
    list: Vec<Box<TransactionRule>>,
}
impl TransactionRule for TransactionRuleCollection {
    fn is_satisfied(&self, transaction: &Transaction) -> bool {
        self.list.iter().all(|Rule| Rule.is_satisfied(transaction)) //TODO question: do we use &Rule?
    }
}

/// The common trait for block rules
pub trait BlockRule {
    fn result(&self, block: &Block) -> BlockRuleResult;
}
/// Return of BlockRule result function
pub enum BlockRuleResult {
    False,                                            // If a logical check fails
    MissingReferencesInDBandBC(Vec<H256>, Vec<H256>), // Block references are not present in DB and blockchain respectively
    True,                                             //Else: the block passes all the checks
}

/// Common Rule for every type of block
pub struct IsNew {
    blockchain: Arc<Mutex<BlockChain>>,
    block_db: Arc<BlockDatabase>,
}

impl BlockRule for IsNew {
    fn result(&self, block: &Block) -> BlockRuleResult {
        let block_inner =
            data_availability::get_available_block(block.hash(), &self.blockchain, &self.block_db);
        match block_inner {
            data_availability::BlockDataAvailability::NotInDB => {
                return BlockRuleResult::True;
            }
            data_availability::BlockDataAvailability::NotInBlockchain => {
                unimplemented!("What should we do here?");
            }
            data_availability::BlockDataAvailability::Block(_) => {
                return BlockRuleResult::False;
            }
        }

        unimplemented!();
    }
}

/// Collection of transaction rules
#[derive(Default)]
pub struct BlockRuleCollection {
    list: Vec<Box<BlockRule>>,
}
impl BlockRule for BlockRuleCollection {
    fn result(&self, block: &Block) -> BlockRuleResult {
        unimplemented!();
        //        iself.list.iter().all(|Rule| Rule.result(block))
    }
}
