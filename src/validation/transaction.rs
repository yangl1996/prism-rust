use std::cell::RefCell;
use crate::crypto::hash::Hashable;
use crate::transaction::{CoinId, Transaction, Address};
use crate::utxodb::UtxoDatabase;

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

pub fn check_num_authorizations(transaction: &Transaction) -> bool {
    let mut owners: Vec<Address> = transaction.input.iter().map(|x|x.owner).collect();
    owners.sort_unstable();
    owners.dedup();

    // check that all owners have signed the transaction
    if owners.len() != transaction.authorization.len() { return false; }
    let owners_match = owners.iter().zip(transaction.authorization.iter()).all(|(owner, authorization)| {
        let addr: Address = ring::digest::digest(&ring::digest::SHA256, &authorization.pubkey.as_bytes().as_ref()).into();
        addr == *owner
    });
    if !owners_match {
        return false;
    }
    return true;
}
