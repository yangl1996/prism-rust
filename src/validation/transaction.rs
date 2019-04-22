use super::*;
use crate::crypto::hash::Hashable;
use crate::state::{CoinId, UTXODatabase};
use crate::transaction::Transaction;
use std::sync::{Arc, Mutex};

/// Checks that input and output are non-empty
pub struct NonEmptyRule;
impl TransactionRule for NonEmptyRule {
    fn is_satisfied(&self, transaction: &Transaction) -> bool {
        !(transaction.input.is_empty() || transaction.output.is_empty())
    }
}

/// Checks if all the inputs are unspent
pub struct UnspentInputRule {
    state_db: Arc<Mutex<UTXODatabase>>,
}
impl TransactionRule for UnspentInputRule {
    fn is_satisfied(&self, transaction: &Transaction) -> bool {
        let mut utxo_state = self.state_db.lock().unwrap();
        transaction.input.iter().all(|input| {
            utxo_state
                .check(&CoinId {
                    hash: input.hash,
                    index: input.index as usize,
                })
                .unwrap()
        })
    }
}

/// Checks if the input and output values are positive and  if input_sum >= output_sum
pub struct InputOutputValuesRule;
impl TransactionRule for InputOutputValuesRule {
    fn is_satisfied(&self, transaction: &Transaction) -> bool {
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
}

/// Checks if the signatures are correct
pub struct VerifySignatureRule;
impl TransactionRule for VerifySignatureRule {
    fn is_satisfied(&self, transaction: &Transaction) -> bool {
        //Checking if the number of signatures are same as number of inputs
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
}
