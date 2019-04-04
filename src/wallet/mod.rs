use crate::crypto::hash::{H256, Hashable};
use std::collections::{HashMap, HashSet};
use crate::transaction::{Transaction, Input, Output};
use crate::crypto::sign::{PubKey, SecKey, KeyPair, Signature};


#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Coin  {
    /// The 'Input' ds to be included in the 'Transaction' ds when the coin is spent
    input: Input,
    /// The value of the coin
    value: u64,
    /// Used to sign the transaction which spends this coin
    signature: Signature
}

pub struct Wallet {
    /// List of coins which can be spent
    coins: HashSet<Coin>,
    /// List of user keys.
    keys: HashMap<H256, KeyPair>
}

impl Wallet {
    pub fn new() -> Self {
        return Self {coins: HashSet::new(), keys: HashMap::new()};
    }

    pub fn generate_new_key(&mut self) {
        unimplimented!();
    }

    /// Add coins from a transaction
    pub fn add_coins(&mut self, transaction: &Transaction){
        for index in 0..transaction.output.len(){
            self.add_coin(transaction, index); // TODO: Can be parallelized
        }
    }

    /// Add the coin to wallet if the user is a recipient.
    pub fn add_coin(&mut self, transaction: &Transaction, index: usize) {
        let output: Output = transaction.output[index].clone();
        // check coin recipient is in my pubkeys
        if self.keys.contains_key(&output.recipient){
            // Construct coin
            let input = Input{hash: transaction.hash(), index: index as u32};
            let value = output.value;
            let keypair = self.keys.get(&output.recipient);
            // TODO: Generate Signature from keypair field
            let signature = Signature::default();
            let coin = Coin{input, value, signature};
            self.coins.insert(coin);
        }
    }

    /// Removes coin from the wallet. Will be used after spending the coin.
    pub fn remove_coin(&mut self, coin: Coin) {
        self.coins.remove(&coin);
    }

    ///  Returns the sum of values of all the coin in the wallet
    pub fn total_balance(&self) -> u64 {
        self.coins.iter().map(|coin| coin.value).sum::<u64>()
    }

    /// create a transaction using the wallet coins
    pub fn create_transaction(&mut self, recipient: H256, value: u64) -> Option<Transaction> {
        let mut coins: Vec<Coin>= vec![];
        let mut value_sum = 0u64;

        let mut coins_iterator = self.coins.iter();
        // first while, we use by_output rather than by_used_outpoint
        while let Some(coin) = coins_iterator.next() {
            value_sum += coin.value;
            coins.push(coin.clone());

            if value_sum >= value { // if we have enough money in our wallet, create tx
                // 1. Create transaction inputs
                let mut input: Vec<Input>  = vec![];
                let mut signatures: Vec<Signature>  = vec![];

                for coin in coins.iter() { // add inputs to used_outpoint to avoid potential double spend
                    input.push(coin.input.clone());
                    signatures.push(coin.signature.clone());
                    self.remove_coin(coin);
                }
                // 2. Create transaction outputs
                let mut output = vec![Output {recipient, value}];
                if value_sum > value {
                    // the output that transfer remaining value to himself
                    let recipient: H256 = match self.keys.keys().next() {
                        Some(&x) => x ,
                        None => panic!("The wallet has no keys"),
                    };
                    output.push(Output{recipient, value: value_sum - value})
                }

                // 3. TODO: Sign the transaction
                return Some(Transaction {
                    input,
                    output,
                    signatures: vec![],
                });
            }
        }

        return None;
    }


}

//#[cfg(test)]
//pub mod tests {
//    use super::Wallet;
//    use crate::transaction::{Input,Output};
//    use crate::crypto::generator as crypto_generator;
//
//    #[test]
//    pub fn test_wallet_balance() {
//        let hash = crypto_generator::h256();
//        let mut w = Wallet::new(hash.clone());
//        w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: crypto_generator::h256()});
//        assert_eq!(w.total_balance(), 0);
//        w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
//        assert_eq!(w.total_balance(), 10);
//        assert_eq!(w.safe_balance(), 10);
//    }
//
//    #[test]
//    pub fn test_wallet_create() {
//        let hash = crypto_generator::h256();
//        let mut w = Wallet::new(hash.clone());
//        // add 10*10 coins
//        for i in 0..10 {
//            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
//        }
//        assert_eq!(w.total_balance(), 100);
//        let tx = w.create(crypto_generator::h256(), 29);
//        //println!("{:?}", tx);
//        if let Some(tx) = tx {
//            // This transaction should be input(10,10,10) output(29,1)
//            assert_eq!(tx.input.len(),3);
//            assert_eq!(tx.output.len(),2);
//        } else {
//            panic!("transaction creation failed")
//        }
//
//        assert!(w.create(crypto_generator::h256(), 10000).is_none());
//    }
//
//    #[test]
//    pub fn test_wallet_create_2() {
//        let hash = crypto_generator::h256();
//        let mut w = Wallet::new(hash.clone());
//        // add 10*10 coins
//        for i in 0..10 {
//            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
//        }
//        assert_eq!(w.total_balance(), 100);
//        // spend 5*20 coins
//        for i in 0..5 {
//            assert!(w.create(crypto_generator::h256(), 20).is_some());
//        }
//        // balance is still 100, but safe balance is 0
//        assert_eq!(w.total_balance(), 100);
//        assert_eq!(w.safe_balance(), 0);
//        // but all coins are marked as used
//        assert_eq!(w.by_used_outpoint.len(), 10);
//        // but we can still create tx using unsafe coins
//        assert!(w.create(crypto_generator::h256(), 1).is_some());
//    }
//
//    #[test]
//    pub fn test_wallet_create_3() {
//        let hash = crypto_generator::h256();
//        let mut w = Wallet::new(hash.clone());
//        // add 10*10 coins
//        for i in 0..10 {
//            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
//        }
//        assert_eq!(w.total_balance(), 100);
//        // spend 10*10 (although only 5 of 10) coins
//        for i in 0..10 {
//            assert!(w.create(crypto_generator::h256(), 5).is_some());
//        }
//        // balance is still 100, but safe balance is 0
//        assert_eq!(w.total_balance(), 100);
//        assert_eq!(w.safe_balance(), 0);
//        // but all coins are marked as used
//        assert_eq!(w.by_used_outpoint.len(), 10);
//        // but we can still create tx using unsafe coins
//        assert!(w.create(crypto_generator::h256(), 1).is_some());
//    }
//
//    #[test]
//    pub fn test_wallet_update_1() {
//        let hash = crypto_generator::h256();
//        let mut w = Wallet::new(hash.clone());
//        // add 10*10 coins
//        for i in 0..10 {
//            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
//        }
//        assert_eq!(w.total_balance(), 100);
//        // spend 5*20 coins
//        for i in 0..5 {
//            assert!(w.create_update(crypto_generator::h256(), 20).is_some());
//        }
//        // now no coin can be spent
//        assert_eq!(w.total_balance(), 0);
//        assert!(w.create(crypto_generator::h256(), 1).is_none());
//
//    }
//
//    #[test]
//    pub fn test_wallet_update_2() {
//        let hash = crypto_generator::h256();
//        let mut w = Wallet::new(hash.clone());
//        // add 10*10 coins
//        for i in 0..10 {
//            w.insert(Input{hash: crypto_generator::h256(), index: 0}, Output{value: 10, recipient: hash.clone()});
//        }
//        assert_eq!(w.total_balance(), 100);
//        // spend 20*5 coins
//        for i in 0..20 {
//            assert!(w.create_update(crypto_generator::h256(), 5).is_some());
//        }
//        // now no coin can be spent
//        assert_eq!(w.total_balance(), 0);
//        assert!(w.create(crypto_generator::h256(), 1).is_none());
//
//    }
//}

