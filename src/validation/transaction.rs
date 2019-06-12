use std::cell::RefCell;
use crate::crypto::hash::Hashable;
use crate::transaction::{CoinId, Transaction, Address};
use crate::utxodb::UtxoDatabase;
use crate::crypto::sign::Signable;

/// Checks that input and output are non-empty
pub fn check_non_empty(transaction: &Transaction) -> bool {
    !(transaction.input.is_empty() || transaction.output.is_empty())
}

/// Checks that input and output value is not 0
pub fn check_non_zero(transaction: &Transaction) -> bool {
    !(transaction.input.iter().any(|x|x.value==0) || transaction.output.iter().any(|x|x.value==0) )
}

/// Checks if input_sum >= output_sum
pub fn check_sufficient_input(transaction: &Transaction) -> bool {
    let input_sum: u64 = transaction.input.iter().map(|x|x.value).sum();
    let output_sum: u64 = transaction.output.iter().map(|x|x.value).sum();
    input_sum >= output_sum
}

pub fn check_signature(transaction: &Transaction) -> bool {
    let mut owners: Vec<Address> = transaction.input.iter().map(|x|x.owner).collect();
    owners.sort_unstable();
    owners.dedup();
    if owners.len() != transaction.authorization.len() { return false; }
    owners.iter().zip(transaction.authorization.iter()).all(|(owner, authorization)| {
        authorization.pubkey.hash() == *owner && transaction.verify(&authorization.pubkey, &authorization.signature)
    })
}
