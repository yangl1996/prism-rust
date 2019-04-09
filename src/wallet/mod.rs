use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign::{KeyPair, Signature};
use crate::miner::memory_pool::MemoryPool;
use crate::miner::miner::ContextUpdateSignal;
use crate::transaction::{Input, Output, Transaction};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use crate::state::{UTXO, CoinId};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Coin {
    utxo: UTXO,
    recipient: H256,
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

pub enum WalletError {
    InsufficientCoin,
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
        let hash = tx.hash();// compute hash here, and below inside Input we don't have to compute again (we just copy)
        for (idx, output) in tx.output.iter().enumerate() {
            if self.keys.contains_key(&output.recipient) {
                let input = CoinId {
                    hash,
                    index: idx as u32,
                };
                let coin = Coin {
                    utxo: UTXO {coin_id: input, value: output.value},
                    recipient: output.recipient,
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
        self.coins.iter().map(|coin| coin.utxo.value).sum::<u64>()
    }

    /// create a transaction using the wallet coins
    fn create_transaction(&mut self, recipient: H256, value: u64) -> Option<Transaction> {
        let mut coins: Vec<Coin> = vec![];
        let mut value_sum = 0u64;

        // iterate thru our wallet
        for coin in self.coins.iter() {
            value_sum += coin.utxo.value;
            coins.push(coin.clone()); // coins that will be used for this transaction

            if value_sum >= value {
                // if we have enough money in our wallet, create tx
                // create transaction inputs
                let input = coins.iter().map(|c|c.utxo.coin_id.clone()).collect();
                // create the output
                let mut output = vec![Output { recipient, value }];
                if value_sum > value {
                    // transfer the remaining value back to self
                    let recipient: H256 = match self.keys.keys().next() {
                        Some(&x) => x,
                        None => panic!("The wallet has no keys"),
                    };
                    output.push(Output {
                        recipient,
                        value: value_sum - value,
                    })
                }

                // remove used coin from wallet
                for c in &coins {
                    self.remove_coin(c);
                }

                // TODO: sign the transaction use coins
                return Some(Transaction {
                    input,
                    output,
                    signatures: vec![],
                });
            }
        }
        return None;
    }

    pub fn send_coin(&mut self, recipient: H256, value: u64) -> Result<(),WalletError> {
        let txn = self.create_transaction(recipient, value);
        let txn = match txn {
            Some(t) => t,
            None => {
                return Err(WalletError::InsufficientCoin);
            }
        };
        let mut mempool = self.mempool.lock().unwrap();
        mempool.insert(txn);
        drop(mempool);
        self.context_update_chan
            .send(ContextUpdateSignal::NewContent).unwrap();
        return Ok(());
    }
}

#[cfg(test)]
pub mod tests {
    use super::Wallet;
    use crate::transaction::{Transaction,Output};
    use crate::crypto::generator as crypto_generator;
    use crate::miner::memory_pool::MemoryPool;
    use std::sync::{mpsc, Arc, Mutex};
    use crate::miner::miner::ContextUpdateSignal;

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
        for i in 0..5 {
            ctx_update_source.recv().unwrap();
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
        for i in 0..5 {
            ctx_update_source.recv().unwrap();
        }
    }
}
