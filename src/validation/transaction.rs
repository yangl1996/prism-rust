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

pub struct AllValidator {
    list: Vec<Box<TransactionValidator>>,
}

impl TransactionValidator for AllValidator {
    fn is_valid(&self, transaction: &Transaction) -> bool {
        self.list.iter().all(|validator|validator.is_valid(transaction))//TODO question: do we use &validator?
    }
}

#[cfg(test)]

pub mod tests {
    use super::{NonEmptyValidator, TransactionValidator};
    use crate::transaction::transaction_builder::TransactionBuilder;
    use crate::validation::transaction::AllValidator;

    #[test]
    fn test_allvalidator() {
        let v = AllValidator { list:vec![Box::new(NonEmptyValidator{})]};
        assert!(v.is_valid(&TransactionBuilder::random_transaction_builder().into()));
    }
}