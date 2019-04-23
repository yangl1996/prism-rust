use super::*;
use crate::crypto::hash::Hashable;
use crate::state::{CoinId, UTXODatabase};
use crate::transaction::Transaction;
use std::sync::{Arc, Mutex};

/// Checks that input and output are non-empty
pub fn check_non_empty(transaction: &Transaction) -> bool {
    !(transaction.input.is_empty() || transaction.output.is_empty())
}

/// Checks if all the inputs are unspent
pub fn check_input_unspent(transaction: &Transaction, utxodb: &UTXODatabase) -> bool {
    transaction.input.iter().all(|input| {
        utxodb
            .check(&CoinId {
                hash: input.hash,
                index: input.index,
            })
        .unwrap()
    })
}

/// Checks if the input and output values are positive and if input_sum >= output_sum
pub fn check_sufficient_input(transaction: &Transaction) -> bool {
    let mut input_sum = 0;
    for input in transaction.input.iter() {
        if input.value < 0 {
            return false;
        }
        input_sum += input.value;
    }
    let mut output_sum = 0;
    for output in transaction.output.iter() {
        if output.value < 0 {
            return false;
        }
        output_sum += output.value;
    }
    return input_sum >= output_sum;
}

pub fn check_signature(transaction: &Transaction) -> bool {
    // TODO: get a set of unique addresses
    // Checking if the number of signatures are same as number of inputs
    if transaction.input.len() != transaction.signatures.len() {
        return false;
    }

    //Checking if the recepient hash = signature pubkey hash
    for index in 0..transaction.input.len() {
        let input = &transaction.input[index];
        let signature = &transaction.signatures[index];
        if input.recipient != signature.pubkey.hash() {
            return false;
        }
    }

    //Verify each signature
    let unsigned_transaction = Transaction {
        input: transaction.input.clone(),
        output: transaction.output.clone(),
        signatures: vec![],
    };
    let msg = bincode::serialize(&unsigned_transaction).unwrap();
    transaction
        .signatures
        .iter()
        .all(|signature| signature.pubkey.verify(&msg, &signature.signature))
}
