/*
Validation for blocks and transactions.
*/

use crate::crypto::hash::Hashable;
use crate::state::{CoinId, UTXODatabase};
use crate::transaction::Transaction;
use std::sync::{Arc, Mutex};

pub trait TransactionValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool;
}

/// Checks that input and output are non-empty
pub struct NonEmptyValidator;
impl TransactionValidator for NonEmptyValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool {
        !(transaction.input.is_empty() || transaction.output.is_empty())
    }
}

/// Checks if all the inputs are unspent
pub struct UnspentInputValidator {
    state_db: Arc<Mutex<UTXODatabase>>,
}
impl TransactionValidator for UnspentInputValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool {
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
pub struct InputOutputValuesValidator;
impl TransactionValidator for InputOutputValuesValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool {
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
pub struct CheckSignatureValidator;
impl TransactionValidator for CheckSignatureValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool {
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

#[derive(Default)]
pub struct ValidatorCollection {
    list: Vec<Box<TransactionValidator>>,
}

impl TransactionValidator for ValidatorCollection {
    fn is_valid(&self, transaction: &Transaction) -> bool {
        self.list
            .iter()
            .all(|validator| validator.is_valid(transaction)) //TODO question: do we use &validator?
    }
}

impl ValidatorCollection {
    pub fn new(list: Vec<Box<TransactionValidator>>) -> Self {
        Self { list }
    }
}
#[cfg(test)]

// TODO: Add more tests
pub mod tests {
    use super::{NonEmptyValidator, TransactionValidator};
    use crate::transaction::generator;
    use crate::validation::transaction::ValidatorCollection;

    #[test]
    fn test_validator() {
        let v = ValidatorCollection::new(vec![Box::new(NonEmptyValidator {})]);
        assert!(v.is_valid(&generator::random().into()));
    }
}
