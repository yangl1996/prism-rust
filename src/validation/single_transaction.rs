/*
Validation for blocks and transactions.
*/
use crate::transaction::Transaction;
pub trait TransactionValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool;
}

pub struct NonEmptyValidator;

impl TransactionValidator for NonEmptyValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool {
        !(transaction.input.is_empty() || transaction.input.is_empty())
    }
}

pub struct SignatureValidator;

impl TransactionValidator for SignatureValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool {
        unimplemented!()
        //transaction.input.len() == transaction.signatures.len()
    }
}

#[derive(Default)]
pub struct ValidatorCollection {
    list: Vec<Box<TransactionValidator>>,
}

impl TransactionValidator for ValidatorCollection {
    fn is_valid(&self, transaction: &Transaction) -> bool {
        self.list.iter().all(|validator|validator.is_valid(transaction))//TODO question: do we use &validator?
    }
}

impl ValidatorCollection {
    pub fn new(list: Vec<Box<TransactionValidator>>) -> Self {
        Self { list }
    }
}
#[cfg(test)]

pub mod tests {
    use super::{NonEmptyValidator, TransactionValidator};
    use crate::transaction::generator;
    use crate::validation::single_transaction::ValidatorCollection;

    #[test]
    fn test_validator() {
        let v = ValidatorCollection::new(vec![Box::new(NonEmptyValidator{})]);
        assert!(v.is_valid(&generator::random_transaction_builder().into()));
    }
}