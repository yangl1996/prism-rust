/*
Validation for blocks and transactions.
*/
use crate::transaction::Transaction;
pub trait TransactionValidator {
    fn is_valid(transaction: &Transaction) -> bool;
}

pub struct NonEmptyValidator;

impl TransactionValidator for NonEmptyValidator {
    fn is_valid(transaction: &Transaction) -> bool {
        !(transaction.input.is_empty() || transaction.input.is_empty())
    }
}

pub struct SignatureValidator;

impl TransactionValidator for SignatureValidator {
    fn is_valid(transaction: &Transaction) -> bool {
        unimplemented!()
        //transaction.input.len() == transaction.signatures.len()
    }
}

#[cfg(test)]

pub mod tests {
    use super::{NonEmptyValidator, TransactionValidator};
    use crate::transaction::transaction_builder::TransactionBuilder;

    #[test]
    fn test_nonemptyvalidator() {
        assert!(NonEmptyValidator::is_valid(&TransactionBuilder::random_transaction_builder().into()));
    }
}