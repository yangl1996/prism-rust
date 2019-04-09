use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign::{KeyPair, Signature};
use crate::miner::memory_pool::MemoryPool;
use crate::miner::miner::ContextUpdateSignal;
use crate::transaction::{Input, Output, Transaction};
use log::error;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Coin {
    input: Input,
    output: Output,
}

pub struct Wallet {
    /// List of coins which can be spent
    coins: HashSet<Coin>,
    /// List of user keys
    keys: HashMap<H256, KeyPair>,
    /// Channel to notify the miner about context update
    context_update_chan: mpsc::Sender<ContextUpdateSignal>,
    /// Pool of unmined transactions
    mempool: Arc<Mutex<MemoryPool>>,
}

impl Wallet {
    pub fn new(
        mempool: &Arc<Mutex<MemoryPool>>,
        ctx_update_sink: mpsc::Sender<ContextUpdateSignal>,
    ) -> Self {
        return Self {
            coins: HashSet::new(),
            keys: HashMap::new(),
            context_update_chan: ctx_update_sink,
            mempool: Arc::clone(mempool),
        };
    }

    pub fn generate_new_key(&mut self) {
        unimplemented!();
    }

    pub fn add_key(&mut self, hash: H256) {
        // TODO: this function does not take a real key for now
        self.keys.insert(hash, KeyPair::default());
    }

    /// Add coins in a transaction that are destined to us
    pub fn add_transaction(&mut self, tx: &Transaction) {
        let hash = tx.hash();
        for (idx, output) in tx.output.iter().enumerate() {
            if self.keys.contains_key(&output.recipient) {
                let input = Input {
                    hash,
                    index: idx as u32,
                };
                let output = output.clone();
                let coin = Coin {
                    input: input,
                    output: output,
                };
                self.coins.insert(coin);
            }
        }
    }

    /// Removes coin from the wallet. Will be used after spending the coin.
    pub fn remove_coin(&mut self, coin: &Coin) {
        self.coins.remove(coin);
    }

    ///  Returns the sum of values of all the coin in the wallet
    pub fn balance(&self) -> u64 {
        self.coins.iter().map(|coin| coin.output.value).sum::<u64>()
    }

    /// create a transaction using the wallet coins
    fn create_transaction(&mut self, recipient: H256, value: u64) -> Option<Transaction> {
        let mut coins: Vec<Coin> = vec![];
        let mut value_sum = 0u64;

        // iterate thru our wallet
        for coin in self.coins.iter().cloned() {
            // TODO: can we remove cloned here?
            value_sum += coin.output.value;
            coins.push(coin.clone()); // coins that will be used for this transaction

            if value_sum >= value {
                // if we have enough money in our wallet, create tx
                // first, create transaction inputs
                let mut input: Vec<Input> = vec![];
                let _signatures: Vec<Signature> = vec![];

                for c in coins {
                    input.push(c.input.clone());
                    self.remove_coin(&c);
                }

                // create the output
                let mut output = vec![Output { recipient, value }];
                if value_sum > value {
                    // transfer the remaining value back to ourself
                    let recipient: H256 = match self.keys.keys().next() {
                        Some(&x) => x,
                        None => panic!("The wallet has no keys"),
                    };
                    output.push(Output {
                        recipient,
                        value: value_sum - value,
                    })
                }

                // TODO: sign the transaction
                return Some(Transaction {
                    input,
                    output,
                    signatures: vec![],
                });
            }
        }
        return None;
    }

    pub fn send_coin(&mut self, recipient: H256, value: u64) {
        let txn = self.create_transaction(recipient, value);
        let txn = match txn {
            Some(t) => t,
            None => {
                // TODO: error handling
                error!("Insufficient wallet balance.");
                return;
            }
        };
        let mut mempool = self.mempool.lock().unwrap();
        mempool.insert(txn);
        drop(mempool);
        self.context_update_chan
            .send(ContextUpdateSignal::NewContent)
            .unwrap();
        return;
    }
}

#[cfg(test)]
pub mod tests {
    use super::Wallet;
    use crate::transaction::{Transaction,Output};
    use crate::crypto::generator as crypto_generator;
    use crate::miner::memory_pool::MemoryPool;
    use std::sync::{mpsc, Arc, Mutex};

    #[test]
    pub fn test_balance() {
        let (ctx_update_sink, ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(&pool, ctx_update_sink);
        w.add_key(hash.clone());
        assert_eq!(w.balance(), 0);
    }
    #[test]
    pub fn test_add_transaction() {
        let (ctx_update_sink, ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(&pool, ctx_update_sink);
        w.add_key(hash.clone());
        let mut output: Vec<Output> = vec![];
        for i in 0..10 {
            output.push(Output{value: 10, recipient: hash.clone()});
        }
        let tx1 = Transaction {
                    input: vec![],
                    output,
                    signatures: vec![],
                };
        w.add_transaction(&tx1);
        assert_eq!(w.balance(), 100);
    }

    #[test]
    pub fn test_send_coin() {
        let (ctx_update_sink, ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(&pool, ctx_update_sink);
        w.add_key(hash.clone());
        let mut output: Vec<Output> = vec![];
        for i in 0..10 {
            output.push(Output{value: 10, recipient: hash.clone()});
        }
        let tx1 = Transaction {
                    input: vec![],
                    output,
                    signatures: vec![],
                };
        w.add_transaction(&tx1);
        // now we have 10*10 coins, we try to spend them
        for i in 0..5 {
            w.send_coin(crypto_generator::h256(), 20);
        }
        assert_eq!(w.balance(), 0);
        let m = pool.lock().unwrap();
        let txs: Vec<Transaction> = m.get_transactions(5).iter().map(|entry|entry.transaction.clone()).collect();
        drop(m);
        assert_eq!(txs.len(), 5);
        for tx in &txs {
            println!("{:?}",tx);
        }
    }

    #[test]
    pub fn test_send_coin_2() {
        let (ctx_update_sink, ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let hash = crypto_generator::h256();
        let mut w = Wallet::new(&pool, ctx_update_sink);
        w.add_key(hash.clone());
        let mut output: Vec<Output> = vec![];
        for i in 0..10 {
            output.push(Output{value: 10, recipient: hash.clone()});
        }
        let tx1 = Transaction {
                    input: vec![],
                    output,
                    signatures: vec![],
                };
        w.add_transaction(&tx1);
        // now we have 10*10 coins, we try to spend them
        for i in 0..5 {
            w.send_coin(crypto_generator::h256(), 19);
        }
        assert_eq!(w.balance(), 0);
        let m = pool.lock().unwrap();
        let txs: Vec<Transaction> = m.get_transactions(5).iter().map(|entry|entry.transaction.clone()).collect();
        drop(m);
        assert_eq!(txs.len(), 5);
        for tx in &txs {
            println!("{:?}",tx);
        }
    }
/*
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
*/
}
