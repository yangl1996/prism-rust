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
    pubkey_hash: H256,
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

#[derive(Debug)]
pub enum WalletError {
    InsufficientMoney,
    MissingKey,
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
                    pubkey_hash: output.recipient,
                };
                self.coins.insert(coin);
            }
        }
    }

    /// Removes coin from the wallet. Will be used after spending the coin.
    pub fn remove_coin(&mut self, coin: &Coin) {
        self.coins.remove(coin);
    }

    /// Returns the sum of values of all the coin in the wallet
    pub fn balance(&self) -> u64 {
        self.coins.iter().map(|coin| coin.utxo.value).sum::<u64>()
    }

    /// create a transaction using the wallet coins
    fn create_transaction(&mut self, recipient: H256, value: u64) -> Result<Transaction,WalletError> {
        let mut coins_to_use: Vec<Coin> = vec![];
        let mut value_sum = 0u64;

        // iterate thru our wallet
        for coin in self.coins.iter() {
            value_sum += coin.utxo.value;
            coins_to_use.push(coin.clone()); // coins that will be used for this transaction
            if value_sum >= value {// if we already have enough money, break
                break;
            }
        }
        if value_sum < value {
            // we don't have enough money in wallet
            return Err(WalletError::InsufficientMoney);
        }
        // if we have enough money in our wallet, create tx
        // create transaction inputs
        let input = coins_to_use.iter().map(|c|c.utxo.coin_id.clone()).collect();
        // create the output
        let mut output = vec![Output { recipient, value }];
        if value_sum > value {
            // transfer the remaining value back to self
            let recipient: H256 = match self.keys.keys().next() {
                Some(&x) => x,
                None => return Err(WalletError::MissingKey),
            };
            output.push(Output {recipient, value: value_sum - value});
        }

        // remove used coin from wallet
        for c in &coins_to_use {
            self.remove_coin(c);
        }

        // TODO: sign the transaction use coins
        return Ok(Transaction {
            input,
            output,
            signatures: vec![],
        });
    }

    pub fn send_coin(&mut self, recipient: H256, value: u64) -> Result<(),WalletError> {
        let txn = self.create_transaction(recipient, value)?;
        let mut mempool = self.mempool.lock().unwrap();// we should use handler to work with mempool
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
    use crate::crypto::hash::H256;

    fn new_wallet_pool_receiver() -> (Wallet, Arc<Mutex<MemoryPool>>, mpsc::Receiver<ContextUpdateSignal>) {
        let (ctx_update_sink, ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let w = Wallet::new(&pool, ctx_update_sink);
        return (w,pool,ctx_update_source);
    }
    fn transaction_10_10(recipient: &H256) -> Transaction {
        let mut output: Vec<Output> = vec![];
        for i in 0..10 {
            output.push(Output{value: 10, recipient: recipient.clone()});
        }
        return Transaction {
            input: vec![],
            output,
            signatures: vec![],
        };
    }
    #[test]
    pub fn test_balance() {
        let (mut w,pool,ctx_update_source) = new_wallet_pool_receiver();
        let hash = crypto_generator::h256();
        w.add_key(hash.clone());
        assert_eq!(w.balance(), 0);
    }
    #[test]
    pub fn test_add_transaction() {
        let (mut w,pool,ctx_update_source) = new_wallet_pool_receiver();
        let hash = crypto_generator::h256();
        w.add_key(hash.clone());
        w.add_transaction(&transaction_10_10(&hash));
        assert_eq!(w.balance(), 100);
    }

    #[test]
    pub fn test_send_coin() {
        let (mut w,pool,ctx_update_source) = new_wallet_pool_receiver();
        let hash = crypto_generator::h256();
        w.add_key(hash.clone());
        w.add_transaction(&transaction_10_10(&hash));
        // now we have 10*10 coins, we try to spend them
        for i in 0..5 {
            assert!(w.send_coin(crypto_generator::h256(), 20).is_ok());
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
        let (mut w,pool,ctx_update_source) = new_wallet_pool_receiver();
        let hash = crypto_generator::h256();
        w.add_key(hash.clone());
        w.add_transaction(&transaction_10_10(&hash));
        // now we have 10*10 coins, we try to spend them
        for i in 0..5 {
            assert!(w.send_coin(crypto_generator::h256(), 19).is_ok());
        }
        assert_eq!(w.balance(), 0);
        let m = pool.lock().unwrap();
        let txs: Vec<Transaction> = m.get_transactions(5).iter().map(|entry|entry.transaction.clone()).collect();
        drop(m);
        assert_eq!(txs.len(), 5);
        for tx in &txs {// for test, just add new tx into this wallet
            println!("{:?}",tx);
            w.add_transaction(tx);
        }
        assert_eq!(w.balance(), 5);//10*10-5*19=5 remaining
        for i in 0..5 {
            ctx_update_source.recv().unwrap();
        }
    }

    #[test]
    pub fn test_send_coin_fail() {
        let (mut w,pool,ctx_update_source) = new_wallet_pool_receiver();
        let hash = crypto_generator::h256();
        w.add_key(hash.clone());
        w.add_transaction(&transaction_10_10(&hash));
        // now we have 10*10 coins, we try to spend 101
        assert!(w.send_coin(crypto_generator::h256(), 101).is_err());
        // we try to spend 20 6 times, the 6th time should have err
        for i in 0..5 {
            assert!(w.send_coin(crypto_generator::h256(), 20).is_ok());
        }
        assert!(w.send_coin(crypto_generator::h256(), 20).is_err());
        assert_eq!(w.balance(), 0);
    }
}
