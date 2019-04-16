use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign::{KeyPair, PubKey, Signature};
use crate::handler;
use crate::miner::memory_pool::MemoryPool;
use crate::miner::miner::ContextUpdateSignal;
use crate::state::{CoinData, CoinId, UTXO};
use crate::transaction::{Input, Output, Signature as PubkeySignature, Transaction};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub type Result<T> = std::result::Result<T, WalletError>;

pub type Coin = UTXO;

// one potential problem: if another program has the same keypair, then he may spend a coin, but this wallet don't know it's spend.
// another problem is concurrency, it seems this wallet can only be run single-threaded.
// so this wallet should just be used in experiment to generate transactions single-threaded.
/// A data structure to maintain key pairs and their coins, and to generate transactions.
pub struct Wallet {
    /// List of coins which can be spent
    coins: HashMap<CoinId, CoinData>,
    /// List of user keys
    keypairs: HashMap<H256, KeyPair>,
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
            coins: HashMap::new(),
            keypairs: HashMap::new(),
            context_update_chan: ctx_update_sink,
            mempool: Arc::clone(mempool),
        };
    }

    // someone pay to public key A first, then I coincidentally generate A, I will NOT receive the payment
    /// Generate a new key pair
    pub fn generate_keypair(&mut self) {
        let keypair = KeyPair::new();
        self.keypairs.insert(keypair.public_key().hash(), keypair);
    }

    /// Get one pubkey from this wallet
    pub fn get_pubkey(&self) -> Result<PubKey> {
        if let Some(keypair) = self.keypairs.values().next() {
            return Ok(keypair.public_key());
        }
        Err(WalletError::MissingKey)
    }

    // this method doesn't compute hash again. we could get pubkey then compute the hash but that compute hash again!
    /// Get one pubkey's hash from this wallet
    pub fn get_pubkey_hash(&self) -> Result<H256> {
        if let Some(&pubkey_hash) = self.keypairs.keys().next() {
            return Ok(pubkey_hash);
        }
        Err(WalletError::MissingKey)
    }

    /// Add coin to wallet
    fn insert_coin(&mut self, coin: Coin) {
        self.coins.insert(coin.coin_id, coin.coin_data);
    }

    /// Remove the spent coins belong to us in a transaction. Add coins in a transaction that are destined to us
    pub fn receive(&mut self, tx: &Transaction) {
        let hash: H256 = tx.hash(); // compute hash here, and below inside Input we don't have to compute again (we just copy)
        for input in tx.input.iter() {
            self.delete_coin(input);
        }
        for (idx, output) in tx.output.iter().enumerate() {
            if self.keypairs.contains_key(&output.recipient) {
                let coin_id = CoinId {
                    hash,
                    index: idx as u32,
                };
                let coin_data = CoinData {
                    value: output.value,
                    recipient: output.recipient,
                };
                let coin = Coin { coin_id, coin_data };
                self.insert_coin(coin);
            }
        }
    }

    /// Removes coin from the wallet. Will be used after the tx is confirmed and the coin is spent. Also used in rollback
    /// If the coin was in, it is removed. If not, this fn does NOT panic/error.
    fn delete_coin(&mut self, coin_id: &CoinId) {
        self.coins.remove(coin_id);
    }

    /// If Rollback on ledger happens, we need to rollback the wallet. The reverse of receive.
    pub fn rollback(&mut self, tx: &Transaction) {
        unimplemented!();
    }
    /// Returns the sum of values of all the coin in the wallet
    pub fn balance(&self) -> u64 {
        self.coins
            .values()
            .map(|coin_data| coin_data.value)
            .sum::<u64>()
    }

    /// create a transaction using the wallet coins
    fn create_transaction(&mut self, recipient: H256, value: u64) -> Result<Transaction> {
        let mut coins_to_use: Vec<CoinId> = vec![];
        let mut value_sum = 0u64;

        // iterate thru our wallet
        for (coin_id, coin_data) in self.coins.iter() {
            value_sum += coin_data.value;
            coins_to_use.push(coin_id.clone()); // coins that will be used for this transaction
            if value_sum >= value {
                // if we already have enough money, break
                break;
            }
        }
        if value_sum < value {
            // we don't have enough money in wallet
            return Err(WalletError::InsufficientMoney);
        }
        // if we have enough money in our wallet, create tx
        // create transaction inputs
        let input = coins_to_use.clone(); //since Input DS may change, this line may change
                                          // create the output
        let mut output = vec![Output { recipient, value }];
        if value_sum > value {
            // transfer the remaining value back to self
            let recipient = self.get_pubkey_hash()?;
            output.push(Output {
                recipient,
                value: value_sum - value,
            });
        }

        // remove used coin from wallet
        for c in &coins_to_use {
            self.delete_coin(c);
        }

        let unsigned = Transaction {
            input,
            output,
            signatures: vec![],
        };
        //        let mut signatures = vec![];
        //        let message = bincode::serialize(&unsigned).unwrap();
        //        for keypair in coins_to_use.iter().map(|c|self.keypairs.get(&c.pubkey_hash).unwrap()) {
        //            let signature = keypair.sign(&message);
        //            signatures.push(PubkeySignature{pubkey: keypair.public_key(), signature});
        //        }

        Ok(unsigned)
    }

    /// pay to a recipient some value of money, note that the resulting transaction may not be confirmed
    pub fn pay(&mut self, recipient: H256, value: u64) -> Result<H256> {
        let tx = self.create_transaction(recipient, value)?;
        let hash = tx.hash();
        handler::new_transaction(tx, &self.mempool);
        self.context_update_chan
            .send(ContextUpdateSignal::NewContent)
            .unwrap();
        //return tx hash, later we can confirm it in ledger
        Ok(hash)
    }

    // only for test, how to set pub functions just for test?
    pub fn get_coin_id(&self) -> Vec<CoinId> {
        self.coins.keys().cloned().collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::Wallet;
    use crate::crypto::generator as crypto_generator;
    use crate::crypto::hash::{Hashable, H256};
    use crate::miner::memory_pool::MemoryPool;
    use crate::miner::miner::ContextUpdateSignal;
    use crate::transaction::{Output, Transaction};
    use std::sync::{mpsc, Arc, Mutex};

    fn new_wallet_pool_receiver_keyhash() -> (
        Wallet,
        Arc<Mutex<MemoryPool>>,
        mpsc::Receiver<ContextUpdateSignal>,
        H256,
    ) {
        let (ctx_update_sink, ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let mut w = Wallet::new(&pool, ctx_update_sink);
        w.generate_keypair();
        let h: H256 = w.get_pubkey_hash().unwrap();
        return (w, pool, ctx_update_source, h);
    }
    fn transaction_10_10(recipient: &H256) -> Transaction {
        let mut output: Vec<Output> = vec![];
        for i in 0..10 {
            output.push(Output {
                value: 10,
                recipient: recipient.clone(),
            });
        }
        return Transaction {
            input: vec![],
            output,
            signatures: vec![],
        };
    }
    #[test]
    pub fn test_balance() {
        let (mut w, pool, ctx_update_source, hash) = new_wallet_pool_receiver_keyhash();
        assert_eq!(w.balance(), 0);
    }
    #[test]
    pub fn test_add_transaction() {
        let (mut w, pool, ctx_update_source, hash) = new_wallet_pool_receiver_keyhash();
        w.receive(&transaction_10_10(&hash));
        assert_eq!(w.balance(), 100);
    }

    #[test]
    pub fn test_send_coin() {
        let (mut w, pool, ctx_update_source, hash) = new_wallet_pool_receiver_keyhash();
        w.receive(&transaction_10_10(&hash));
        // now we have 10*10 coins, we try to spend them
        for i in 0..5 {
            assert!(w.pay(crypto_generator::h256(), 20).is_ok());
        }
        assert_eq!(w.balance(), 0);
        let m = pool.lock().unwrap();
        let txs: Vec<Transaction> = m.get_transactions(5).iter().map(|tx| tx.clone()).collect();
        drop(m);
        assert_eq!(txs.len(), 5);
        for tx in &txs {
            println!("{:?}", tx);
        }
        for i in 0..5 {
            ctx_update_source.recv().unwrap();
        }
    }

    #[test]
    pub fn test_send_coin_2() {
        let (mut w, pool, ctx_update_source, hash) = new_wallet_pool_receiver_keyhash();
        w.receive(&transaction_10_10(&hash));
        // now we have 10*10 coins, we try to spend them
        for i in 0..5 {
            assert!(w.pay(crypto_generator::h256(), 19).is_ok());
        }
        assert_eq!(w.balance(), 0);
        let m = pool.lock().unwrap();
        let txs: Vec<Transaction> = m.get_transactions(5).iter().map(|tx| tx.clone()).collect();
        drop(m);
        assert_eq!(txs.len(), 5);
        for tx in &txs {
            // for test, just add new tx into this wallet
            println!("{:?}", tx);
            w.receive(tx);
        }
        assert_eq!(w.balance(), 5); //10*10-5*19=5 remaining
        for i in 0..5 {
            ctx_update_source.recv().unwrap();
        }
    }

    #[test]
    pub fn test_send_coin_fail() {
        let (mut w, pool, ctx_update_source, hash) = new_wallet_pool_receiver_keyhash();
        w.receive(&transaction_10_10(&hash));
        // now we have 10*10 coins, we try to spend 101
        assert!(w.pay(crypto_generator::h256(), 101).is_err());
        // we try to spend 20 6 times, the 6th time should have err
        for i in 0..5 {
            assert!(w.pay(crypto_generator::h256(), 20).is_ok());
        }
        assert!(w.pay(crypto_generator::h256(), 20).is_err());
        assert_eq!(w.balance(), 0);
    }

    #[test]
    pub fn key_missing() {
        let (ctx_update_sink, ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let mut w = Wallet::new(&pool, ctx_update_sink);
        assert!(w.get_pubkey_hash().is_err());
        assert!(w.get_pubkey().is_err());
        assert!(w.pay(crypto_generator::h256(), 1).is_err());
        w.generate_keypair();
        assert!(w.get_pubkey_hash().is_ok());
        assert!(w.get_pubkey().is_ok());
        assert!(w.pay(crypto_generator::h256(), 1).is_err());
    }
}
