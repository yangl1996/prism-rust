use crate::transaction::{Transaction, IndexedTransaction, Output, Input};
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::generator;
use rand::{thread_rng, Rng};

#[derive(Debug, Default, Clone)]
pub struct TransactionBuilder {
    pub transaction: Transaction,
}


impl Into<Transaction> for TransactionBuilder {
    fn into(self) -> Transaction {
        self.transaction
    }
}

impl Into<IndexedTransaction> for TransactionBuilder {
    fn into(self) -> IndexedTransaction {
        IndexedTransaction {
            hash: self.transaction.hash(),
            raw: self.transaction,
        }
    }
}

impl TransactionBuilder {
    pub fn add_output(mut self, value: u64, recipient: H256) -> TransactionBuilder {
        self.transaction.output.push(Output {
            value: value,
            recipient: recipient,
        });
        self
    }

    pub fn set_output(mut self, value: u64, recipient: H256) -> TransactionBuilder {
        self.transaction.output = vec![Output {
            value: value,
            recipient: recipient,
        }];
        self
    }

    pub fn add_input(mut self, hash: H256, index: u32) -> TransactionBuilder {
        self.transaction.input.push(Input {
            hash: hash,
            index: index,
        });
        self
    }

    pub fn set_input(mut self, hash: H256, index: u32) -> TransactionBuilder {
        self.transaction.input = vec![Input {
            hash: hash,
            index: index,
        }];
        self
    }


}

#[cfg(test)]
pub mod tests {
    use super::TransactionBuilder;
    use crate::crypto::hash::H256;

    #[test]
    fn test_transaction_builder() {
        let mut tb = TransactionBuilder::default();
        tb = tb.add_input(H256([0,0]), 0);
        tb = tb.add_output(4, H256([1,1]));
        println!("{:?}",tb);
    }

}