use crate::transaction::Transaction;

use ed25519_dalek::PublicKey;
use ed25519_dalek::Signature;

use std::convert::TryFrom;

/// Checks that input and output are non-empty
pub fn check_non_empty(transaction: &Transaction) -> bool {
    !(transaction.input.is_empty() || transaction.output.is_empty())
}

/// Checks that input and output value is not 0
pub fn check_non_zero(transaction: &Transaction) -> bool {
    !(transaction.input.iter().any(|x| x.value == 0)
        || transaction.output.iter().any(|x| x.value == 0))
}

/// Checks if input_sum >= output_sum
pub fn check_sufficient_input(transaction: &Transaction) -> bool {
    let input_sum: u64 = transaction.input.iter().map(|x| x.value).sum();
    let output_sum: u64 = transaction.output.iter().map(|x| x.value).sum();
    input_sum >= output_sum
}

pub fn check_signature_batch(transactions: &[Transaction]) -> bool {
    let mut raw_messages: Vec<Vec<u8>> = vec![];
    let mut messages: Vec<&[u8]> = vec![];
    let mut signatures: Vec<Signature> = vec![];
    let mut public_keys: Vec<PublicKey> = vec![];

    for (_idx, tx) in transactions.iter().enumerate() {
        let raw_inputs = bincode::serialize(&tx.input).unwrap();
        let raw_outputs = bincode::serialize(&tx.output).unwrap();
        let raw = [&raw_inputs[..], &raw_outputs[..]].concat();
        raw_messages.push(raw);
    }

    for (idx, tx) in transactions.iter().enumerate() {
        for a in &tx.authorization {
            public_keys.push(PublicKey::from_bytes(&a.pubkey).unwrap());
            signatures.push(Signature::try_from(&a.signature[..]).unwrap());
            messages.push(&raw_messages[idx]);
        }
    }

    // TODO: tune the batch size
    match ed25519_dalek::verify_batch(&messages, &signatures, &public_keys) {
        Ok(()) => true,
        Err(_) => false,
    }
}
