// pub mod transaction;
pub mod proposer;

use crate::block::{Block};
use crate::blockchain::{BlockChain};
use crate::crypto::hash::{Hashable,H256};
use std::collections::HashSet;

pub trait Validator<'a> {
    
    fn new(blockchain: &'a BlockChain) -> Self;
    fn is_valid(&self, block: &'a Block) -> bool;
    fn is_duplicate(&self, block: &'a Block) -> bool;
    fn is_empty(&self, block: &'a Block) -> bool;
    fn is_pow_valid(&self, block: &'a Block) -> bool;
    fn is_coinbase_valid(&self, block: &'a Block) -> bool;
}


// impl std::ops::Mul<&[u8; 32]> for [u8; 32] {
//     fn mul(self, input: u16) -> Self {
//         // Multiply byte array by u16
//         let mut buffer: [u8; 32] = [0; 32];
//         // TODO: Fix this!! it is incorrect
//         let number = u128::from_be_bytes(self[0..16]);
//         let output = u128::to_be_bytes(number * input)

//         BigEndian::write_u128(&mut buffer[0..16], output);
//         return buffer;
//     }
// }