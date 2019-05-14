use crate::crypto::hash::Hashable;
use crate::transaction::{CoinId, Transaction};
use crate::utxodb::UtxoDatabase;

/// Checks that input and output are non-empty
pub fn check_non_empty(transaction: &Transaction) -> bool {
    !(transaction.input.is_empty() || transaction.output.is_empty())
}

/// Checks if all the inputs are unspent
pub fn check_input_unspent(transaction: &Transaction, utxodb: &UtxoDatabase) -> bool {
    transaction.input.iter().all(|input| {
        utxodb
            .contains(&CoinId {
                hash: input.coin.hash,
                index: input.coin.index,
            })
            .unwrap()
    })
}

/// Checks if the input and output values are positive and if input_sum >= output_sum
pub fn check_sufficient_input(transaction: &Transaction) -> bool {
    let mut input_sum = 0;
    for input in transaction.input.iter() {
        input_sum += input.value;
    }
    let mut output_sum = 0;
    for output in transaction.output.iter() {
        output_sum += output.value;
    }
    return input_sum >= output_sum;
}

pub fn check_signature(transaction: &Transaction) -> bool {
    // TODO: get a set of unique addresses
    // Checking if the number of signatures are same as number of inputs
    if transaction.input.len() != transaction.authorization.len() {
        return false;
    }

    //Checking if the recepient hash = signature pubkey hash
    for index in 0..transaction.input.len() {
        let input = &transaction.input[index];
        let signature = &transaction.authorization[index];
        if input.owner != signature.pubkey.hash() {
            return false;
        }
    }

    //Verify each signature
    let unsigned_transaction = Transaction {
        input: transaction.input.clone(),
        output: transaction.output.clone(),
        authorization: vec![],
    };
    let msg = bincode::serialize(&unsigned_transaction).unwrap();
    transaction
        .authorization
        .iter()
        .all(|signature| signature.pubkey.verify(&msg, &signature.signature))
}
