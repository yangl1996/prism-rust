use crate::crypto::hash::{H256, Hashable};
use std::collections::{HashMap, HashSet};
use crate::transaction::{Transaction, Input, Output};
use crate::crypto::sign::PubKey;

#[derive(Debug)]
pub struct Wallet {
    /// Transaction outpoint -> coin value and owner. (owner must be this user)
    by_outpoint: HashMap<Input, Output>,
    /// Transaction outpoint that are used to create transaction, but not confirmed
    by_used_outpoint: HashMap<Input, Output>,
    /// The (hashed) public keys of this user
    pubkey_hash: H256,// later should change to pubkeys: Vec<PubKey>,
}

impl Wallet {
    pub fn new(pubkey_hash: H256) -> Self {
        return Self {
            by_outpoint: HashMap::new(),
            by_used_outpoint: HashMap::new(),
            pubkey_hash,
        };
    }

    pub fn insert(&mut self, outpoint: Input, coin: Output) {
        // check coin recipient is in my pubkeys
        if coin.recipient == self.pubkey_hash {
            self.by_outpoint.insert(outpoint, coin);
        }
    }

    pub fn remove(&mut self, outpoint: &Input) {
        self.by_outpoint.remove(outpoint);
        self.by_used_outpoint.remove(outpoint);
    }

    pub fn total_balance(&self) -> u64 {
        self.by_outpoint.values().map(|output|output.value).sum::<u64>()+
            self.by_used_outpoint.values().map(|output|output.value).sum::<u64>()
    }

    pub fn safe_balance(&self) -> u64 {
        self.by_outpoint.values().map(|output|output.value).sum()
    }

    /// create a transaction using my coins
    pub fn create(&mut self, recipient: H256, value: u64) -> Option<Transaction> {
        let mut input: Vec<Input>= vec![];
        let mut value_sum = 0u64;

        let mut outpoint_iter = self.by_outpoint.iter();
        // first while, we use by_output rather than by_used_outpoint
        while let Some((outpoint,coin)) = outpoint_iter.next() {
            value_sum += coin.value;
            input.push(outpoint.clone());
            if value_sum >= value {
                let mut output = vec![Output {recipient, value}];
                if value_sum > value {
                    // the output that transfer remaining value to himself
                    output.push(Output{recipient: self.pubkey_hash.clone(), value: value_sum - value})
                }
                for outpoint in &input { // add inputs to used_outpoint to avoid potential double spend
                    if let Some(coin) = self.by_outpoint.get(outpoint) {
                        self.by_used_outpoint.insert(outpoint.clone(), coin.clone());
                        self.by_outpoint.remove(outpoint);
                    }
                }
                return Some(Transaction {
                    input,
                    output,
                    signatures: vec![],
                });
            }
        }

        outpoint_iter = self.by_used_outpoint.iter();
        // second while, we use by_used_outpoint
        while let Some((outpoint,coin)) = outpoint_iter.next() {
            value_sum += coin.value;
            input.push(outpoint.clone());
            if value_sum >= value {
                let mut output = vec![Output {recipient, value}];
                if value_sum > value {
                    // the output that transfer remaining value to himself
                    output.push(Output{recipient: self.pubkey_hash.clone(), value: value_sum - value})
                }
                for outpoint in &input { // add inputs to used_outpoint to avoid potential double spend
                    if let Some(coin) = self.by_outpoint.get(outpoint) {
                        self.by_used_outpoint.insert(outpoint.clone(), coin.clone());
                        self.by_outpoint.remove(outpoint);
                    }
                }
                return Some(Transaction {
                    input,
                    output,
                    signatures: vec![],
                });
            }
        }

        return None;
    }

    /// create a transaction and just assume it is confirmed immediately so update coins
    pub fn create_update(&mut self, recipient: H256, value: u64) -> Option<Transaction> {
        if let Some(tx) = self.create(recipient, value) {
            for input in &tx.input {
                self.remove(input);
            }
            for (index,output) in tx.output.iter().enumerate() {
                let hash = tx.hash();
                self.insert(Input{hash, index: index as u32}, output.clone() );
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
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(hash.clone());
        w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: crypto_generator::h256()});
        assert_eq!(w.total_balance(), 0);
        w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
        assert_eq!(w.total_balance(), 10);
        assert_eq!(w.safe_balance(), 10);
    }

    #[test]
    pub fn test_wallet_create() {
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(hash.clone());
        // add 10*10 coins
        for i in 0..10 {
            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
        }
        assert_eq!(w.total_balance(), 100);
        let tx = w.create(crypto_generator::h256(), 29);
        //println!("{:?}", tx);
        if let Some(tx) = tx {
            // This transaction should be input(10,10,10) output(29,1)
            assert_eq!(tx.input.len(),3);
            assert_eq!(tx.output.len(),2);
        } else {
            panic!("transaction creation failed")
        }

        assert!(w.create(crypto_generator::h256(), 10000).is_none());
    }

    #[test]
    pub fn test_wallet_create_2() {
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(hash.clone());
        // add 10*10 coins
        for i in 0..10 {
            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
        }
        assert_eq!(w.total_balance(), 100);
        // spend 5*20 coins
        for i in 0..5 {
            assert!(w.create(crypto_generator::h256(), 20).is_some());
        }
        // balance is still 100, but safe balance is 0
        assert_eq!(w.total_balance(), 100);
        assert_eq!(w.safe_balance(), 0);
        // but all coins are marked as used
        assert_eq!(w.by_used_outpoint.len(), 10);
        // but we can still create tx using unsafe coins
        assert!(w.create(crypto_generator::h256(), 1).is_some());
    }

    #[test]
    pub fn test_wallet_create_3() {
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(hash.clone());
        // add 10*10 coins
        for i in 0..10 {
            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
        }
        assert_eq!(w.total_balance(), 100);
        // spend 10*10 (although only 5 of 10) coins
        for i in 0..10 {
            assert!(w.create(crypto_generator::h256(), 5).is_some());
        }
        // balance is still 100, but safe balance is 0
        assert_eq!(w.total_balance(), 100);
        assert_eq!(w.safe_balance(), 0);
        // but all coins are marked as used
        assert_eq!(w.by_used_outpoint.len(), 10);
        // but we can still create tx using unsafe coins
        assert!(w.create(crypto_generator::h256(), 1).is_some());
    }

    #[test]
    pub fn test_wallet_update_1() {
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(hash.clone());
        // add 10*10 coins
        for i in 0..10 {
            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
        }
        assert_eq!(w.total_balance(), 100);
        // spend 5*20 coins
        for i in 0..5 {
            assert!(w.create_update(crypto_generator::h256(), 20).is_some());
        }
        // now no coin can be spent
        assert_eq!(w.total_balance(), 0);
        assert!(w.create(crypto_generator::h256(), 1).is_none());

    }

    #[test]
    pub fn test_wallet_update_2() {
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(hash.clone());
        // add 10*10 coins
        for i in 0..10 {
            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
        }
        assert_eq!(w.total_balance(), 100);
        // spend 20*5 coins
        for i in 0..20 {
            assert!(w.create_update(crypto_generator::h256(), 5).is_some());
        }
        // now no coin can be spent
        assert_eq!(w.total_balance(), 0);
        assert!(w.create(crypto_generator::h256(), 1).is_none());

    }
}

