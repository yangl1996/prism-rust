use crate::crypto::hash::H256;
use std::collections::HashMap;
use crate::transaction::{Transaction, Input, Output};

// TODO: learn from Parity

#[derive(Debug)]
pub struct Wallet {
    /// Transaction outpoint -> coin value and owner. (owner must be this user)
    by_outpoint: HashMap<Input, Output>,
    //pubkeys: Vec<PubKey>,// The public keys of this user
}

impl Wallet {
    pub fn new() -> Self {
        return Self {
            by_outpoint: HashMap::new(),
        };
    }

    pub fn insert(&mut self, outpoint: Input, coin: Output) {
        self.by_outpoint.insert(outpoint, coin);
    }

    pub fn contains(&self, outpoint: &Input) -> bool {
        return self.by_outpoint.contains_key(outpoint);
    }

    pub fn remove(&mut self, outpoint: &Input) {
        self.by_outpoint.remove(outpoint);
    }

    pub fn balance(&self) -> u64 {
        self.by_outpoint.values().map(|output|output.value).sum()
    }

    pub fn create(&self, recipient: H256, value: u64) -> Option<Transaction> {
        let mut input: Vec<Input>= vec![];
        let mut input_output = self.by_outpoint.iter();
        let mut value_sum = 0u64;
        while let Some((i,o)) = input_output.next() {
            value_sum += o.value;
            input.push(i.clone());
            if value_sum >= value {
                return Some(Transaction {
                    input,
                    output: vec![Output {recipient, value}],//TODO: remaining coins to himself
                    signatures: vec![],
                });
            }
        }
        return None;
    }

    pub fn create_remove(&mut self, recipient: H256, value: u64) -> Option<Transaction> {
        if let Some(tx) = self.create(recipient, value) {
            for input in &tx.input {
                self.by_outpoint.remove(input);
            }
            return Some(tx);
        } else { return None; }
    }

}

#[cfg(test)]
pub mod tests {
    use super::Wallet;
    use crate::transaction::{Input,Output};
    use crate::crypto::generator as crypto_generator;

    #[test]
    pub fn test_wallet_balance() {
        let mut w = Wallet::new();
        assert_eq!(w.balance(), 0);
        w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: crypto_generator::h256()});
        w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 100, recipient: crypto_generator::h256()});
        assert_eq!(w.balance(), 110);
    }

    #[test]
    pub fn test_wallet_create() {
        let mut w = Wallet::new();
        assert_eq!(w.balance(), 0);
        for i in 0..10 {
            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 100, recipient: crypto_generator::h256()});
        }
        assert!(w.create(crypto_generator::h256(), 299).is_some());
        assert!(w.create(crypto_generator::h256(), 10000).is_none());
    }

    #[test]
    pub fn test_wallet_create_remove() {
        let mut w = Wallet::new();
        assert_eq!(w.balance(), 0);
        // add 10*100 coins
        for i in 0..10 {
            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 100, recipient: crypto_generator::h256()});
        }
        // spend 5*200 coins
        for i in 0..5 {
            assert!(w.create_remove(crypto_generator::h256(), 200).is_some());
        }
        // now no coin can be spent
        assert!(w.create(crypto_generator::h256(), 1).is_none());
        assert_eq!(w.balance(), 0);
    }
}

