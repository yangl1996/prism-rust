// pub mod transaction;
pub mod proposer;

use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::crypto::hash::{Hashable, H256};
use std::collections::HashSet;

pub trait Validator<'a> {
    fn new(blockchain: &'a BlockChain) -> Self;
    fn is_valid(&self, block: &'a Block) -> bool;
    fn is_duplicate(&self, block: &'a Block) -> bool;
    fn is_empty(&self, block: &'a Block) -> bool;
    fn is_pow_valid(&self, block: &'a Block) -> bool;
    fn is_coinbase_valid(&self, block: &'a Block) -> bool;
}
