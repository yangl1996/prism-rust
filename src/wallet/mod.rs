use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign::{KeyPair, PubKey, Signable};
use crate::handler;
use crate::transaction::{Address, Authorization, CoinId, Input, Output, Transaction};
use bincode::{deserialize, serialize};
use std::sync::{Arc, Mutex};
use std::{error, fmt};
use std::collections::BTreeSet;

pub const COIN_CF: &str = "COIN";
pub const KEYPAIR_CF: &str = "KEYPAIR";

pub type Result<T> = std::result::Result<T, WalletError>;

/// A data structure to maintain key pairs and their coins, and to generate transactions.
pub struct Wallet {
    /// The underlying RocksDB handle.
    handle: rocksdb::DB,
}

#[derive(Debug)]
pub enum WalletError {
    InsufficientMoney,
    ZeroKey,
    MissingKey,
    MemoryPoolCheckFailure,
    DBError(rocksdb::Error),
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WalletError::InsufficientMoney => write!(f, "Insufficient Money"),
            WalletError::ZeroKey => write!(f, "You have 0 key pair"),
            WalletError::MissingKey => write!(f, "No Key Pair correspond to the Address"),
            WalletError::MemoryPoolCheckFailure => write!(
                f,
                "Your transaction has conflict with some tx in memory pool"
            ),
            WalletError::DBError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for WalletError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            WalletError::DBError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<rocksdb::Error> for WalletError {
    fn from(err: rocksdb::Error) -> WalletError {
        WalletError::DBError(err)
    }
}

impl Wallet {
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let coin_cf = rocksdb::ColumnFamilyDescriptor::new(COIN_CF, rocksdb::Options::default());
        let keypair_cf =
            rocksdb::ColumnFamilyDescriptor::new(KEYPAIR_CF, rocksdb::Options::default());
        let mut db_opts = rocksdb::Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        let handle = rocksdb::DB::open_cf_descriptors(&db_opts, path, vec![coin_cf, keypair_cf])?;
        return Ok(Self { handle });
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        rocksdb::DB::destroy(&rocksdb::Options::default(), &path)?;
        return Self::open(path);
    }

    // someone pay to public key A first, then I coincidentally generate A, I will NOT receive the payment
    /// Generate a new key pair
    pub fn generate_keypair(&self) -> Result<()> {
        let cf = self.handle.cf_handle(KEYPAIR_CF).unwrap();
        let keypair = KeyPair::random();
        let k: Address = keypair.public_key().hash();
        let k = serialize(&k).unwrap();
        let v = keypair.pkcs8_bytes;
        self.handle.put_cf(cf, &k, &v)?;
        Ok(())
    }

    /// Get one pubkey from this wallet
    pub fn get_a_pubkey(&self) -> Result<PubKey> {
        let cf = self.handle.cf_handle(KEYPAIR_CF).unwrap();
        let mut iter = self.handle.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        if let Some((_k, v)) = iter.next() {
            let keypair = KeyPair::from_pkcs8(v.to_vec());
            return Ok(keypair.public_key());
        }
        Err(WalletError::ZeroKey)
    }

    // this method doesn't compute hash again. we could get pubkey then compute the hash but that compute hash again!
    /// Get one pubkey's hash from this wallet
    pub fn get_an_address(&self) -> Result<Address> {
        let cf = self.handle.cf_handle(KEYPAIR_CF).unwrap();
        let mut iter = self.handle.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        if let Some((k, _v)) = iter.next() {
            let hash: Address = deserialize(k.as_ref()).unwrap();
            return Ok(hash);
        }
        Err(WalletError::ZeroKey)
    }

    /// Get a public key by its Address
    fn get_keypair(&self, addr: &Address) -> Result<KeyPair> {
        let cf = self.handle.cf_handle(KEYPAIR_CF).unwrap();
        let k = serialize(addr).unwrap();
        if let Some(v) = self.handle.get_cf(cf, &k)? {
            let keypair = KeyPair::from_pkcs8(v.to_vec());
            return Ok(keypair);
        }
        Err(WalletError::MissingKey)
    }

    /// Check if a pubkey's Address belongs to this wallet
    fn contains_address(&self, addr: &Address) -> Result<bool> {
        let cf = self.handle.cf_handle(KEYPAIR_CF).unwrap();
        let k = serialize(addr).unwrap();
        if let Some(_) = self.handle.get_cf(cf, &k)? {
            return Ok(true);
        }
        Ok(false)
    }

    /// Add coin to wallet
    /// Use write batch to keep atomicity
    fn insert_coin_batch(&self, coin: &Input, batch: &mut rocksdb::WriteBatch) -> Result<()> {
        let cf = self.handle.cf_handle(COIN_CF).unwrap();
        let k = serialize(&coin.coin).unwrap();
        let output = Output {
            value: coin.value,
            recipient: coin.owner,
        };
        let v = serialize(&output).unwrap();
        batch.put_cf(cf, &k, &v)?;
        Ok(())
    }

    /// Removes coin from the wallet. Will be used after the tx is confirmed and the coin is spent. Also used in rollback
    /// If the coin was in, it is removed. If not, this fn does NOT panic/error.
    fn delete_coin(&self, coin_id: &CoinId) -> Result<()> {
        let cf = self.handle.cf_handle(COIN_CF).unwrap();
        let k = serialize(coin_id).unwrap();
        self.handle.delete_cf(cf, &k)?;
        Ok(())
    }

    /// Removes coin from the wallet. Will be used after the tx is confirmed and the coin is spent. Also used in rollback
    /// If the coin was in, it is removed. If not, this fn does NOT panic/error.
    /// Use write batch to keep atomicity
    fn delete_coin_batch(&self, coin_id: &CoinId, batch: &mut rocksdb::WriteBatch) -> Result<()> {
        let cf = self.handle.cf_handle(COIN_CF).unwrap();
        let k = serialize(coin_id).unwrap();
        batch.delete_cf(cf, &k)?;
        Ok(())
    }

    /// Update the wallet atomically using a write batch.
    /// Can serve as add or rollback, based on arguments to_delete and to_insert.
    pub fn update(&self, add: &[Input], remove: &[Input]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();
        for coin in remove {
            self.delete_coin_batch(&coin.coin, &mut batch)?;
        }
        for coin in add {
            if let Ok(true) = self.contains_address(&coin.owner) {
                self.insert_coin_batch(coin, &mut batch)?;
            }
        }
        self.handle.write(batch)?;
        Ok(())
    }

    /// Returns the sum of values of all the coin in the wallet
    pub fn balance(&self) -> Result<u64> {
        let cf = self.handle.cf_handle(COIN_CF).unwrap();
        let iter = self.handle.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        let balance = iter
            .map(|(_k, v)| {
                let coin_data: Output = bincode::deserialize(v.as_ref()).unwrap();
                coin_data.value
            })
            .sum::<u64>();
        Ok(balance)
    }

    /// Create a transaction using the wallet coins
    pub fn create_transaction(&self, recipient: Address, value: u64) -> Result<Transaction> {
        let mut coins_to_use: Vec<Input> = vec![];
        let mut value_sum = 0u64;
        let cf = self.handle.cf_handle(COIN_CF).unwrap();
        let iter = self.handle.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        // iterate thru our wallet
        for (k, v) in iter {
            let coin_id: CoinId = bincode::deserialize(k.as_ref()).unwrap();
            let coin_data: Output = bincode::deserialize(v.as_ref()).unwrap();
            value_sum += coin_data.value;
            coins_to_use.push(Input {
                coin: coin_id,
                value: coin_data.value,
                owner: coin_data.recipient,
            }); // coins that will be used for this transaction
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
        // remove used coin from wallet
        for c in coins_to_use.iter() {
            self.delete_coin(&c.coin)?;
        }

        // create transaction inputs
        let input: Vec<Input> = coins_to_use
            .iter()
            .map(|c| Input {
                coin: CoinId {
                    hash: c.coin.hash,
                    index: c.coin.index,
                },
                value: c.value,
                owner: c.owner,
            })
            .collect();
        // create the output
        let mut output = vec![Output { recipient, value }];
        if value_sum > value {
            // transfer the remaining value back to self
            let recipient = self.get_an_address()?;
            output.push(Output {
                recipient,
                value: value_sum - value,
            });
        }

        let unsigned = Transaction {
            input,
            output,
            authorization: vec![],
        };
        let mut authorization = vec![];
        let owner_set: BTreeSet<Address> = coins_to_use.into_iter().map(|input|input.owner).collect();
        for owner in owner_set.iter() {
            let keypair = self.get_keypair(&owner)?;
            authorization.push(Authorization {
                pubkey: keypair.public_key(),
                signature: unsigned.sign(&keypair),
            });
        }

        Ok(Transaction {
            authorization,
            ..unsigned
        })
    }

    // only for test, how to set pub functions just for test?
    fn get_coin_id(&self) -> Vec<CoinId> {
        let cf = self.handle.cf_handle(COIN_CF).unwrap();
        let iter = self
            .handle
            .iterator_cf(cf, rocksdb::IteratorMode::Start)
            .unwrap();
        // iterate thru our wallet
        iter.map(|(k, _v)| {
            let coin_id: CoinId = bincode::deserialize(k.as_ref()).unwrap();
            coin_id
        })
        .collect()
    }
}

/*
#[cfg(test)]
pub mod tests {
    use super::Wallet;
    use crate::crypto::hash::tests::generate_random_hash;
    use crate::crypto::hash::H256;
    use crate::crypto::sign::Signable;
    use crate::miner::memory_pool::MemoryPool;
    use crate::miner::ContextUpdateSignal;
    use crate::transaction::{Output, Transaction};
    use std::sync::{mpsc, Arc, Mutex};
    use rand::RngCore;

    fn new_wallet_pool_receiver_keyhash() -> (
        Wallet,
        Arc<Mutex<MemoryPool>>,
        mpsc::Receiver<ContextUpdateSignal>,
        H256,
    ) {
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let mut w = Wallet::new(std::path::Path::new(&format!("/tmp/walletdb_{}.rocksdb",rand::thread_rng().next_u32())), &pool).unwrap();
        w.generate_keypair().unwrap();
        let h: H256 = w.get_an_address().unwrap();
        return (w, pool, ctx_update_source, h);
    }
    fn transaction_10_10(recipient: &H256) -> Transaction {
        let mut output: Vec<Output> = vec![];
        for _ in 0..10 {
            output.push(Output {
                value: 10,
                recipient: recipient.clone(),
            });
        }
        return Transaction {
            input: vec![],
            output,
            authorization: vec![],
        };
    }

    fn receive(w: &mut Wallet, tx: &Transaction) {
        // test verify of signature before receive
        for auth in tx.authorization.iter() {
            assert!(tx.verify(&auth.pubkey, &auth.signature));
        }
        let (to_delete, to_insert) = to_coinid_and_potential_utxo(tx);
        assert!(w.update(&to_delete, &to_insert).is_ok());
    }
    fn rollback(w: &mut Wallet, tx: &Transaction) {
        let (to_delete, to_insert) = to_rollback_coinid_and_potential_utxo(tx);
        assert!(w.update(&to_delete, &to_insert).is_ok());
    }

    #[test]
    pub fn send_coin() {
        let (mut w, pool, ctx_update_source, hash) = new_wallet_pool_receiver_keyhash();
        assert_eq!(w.balance().unwrap(), 0);
        receive(&mut w, &transaction_10_10(&hash));
        assert_eq!(w.balance().unwrap(), 100);
        // now we have 10*10 coins, we try to spend them
        for _ in 0..5 {
            assert!(w.pay(generate_random_hash(), 19).is_ok());
        }
        assert_eq!(w.balance().unwrap(), 0);
        // we have 0 money, so pay someone 20 coin will fail
        assert!(w.pay(generate_random_hash(), 20).is_err());
        let m = pool.lock().unwrap();
        let txs: Vec<Transaction> = m.get_transactions(5);
        drop(m);
        assert_eq!(txs.len(), 5);
        for _ in 0..5 {
            ctx_update_source.recv().unwrap();
        }
        for tx in &txs {
            // for test, just add new tx into this wallet
            //            println!("{:?}", tx);
            receive(&mut w, &tx);
        }
        assert_eq!(w.balance().unwrap(), 5);
    }

    #[test]
    pub fn key_missing() {
        let (ctx_update_sink, _ctx_update_source) = mpsc::channel();
        let pool = Arc::new(Mutex::new(MemoryPool::new()));
        let mut w = Wallet::new(std::path::Path::new(&format!("/tmp/walletdb_{}.rocksdb",rand::thread_rng().next_u32())), &pool, ctx_update_sink).unwrap();
        assert!(w.get_an_address().is_err());
        assert!(w.get_a_pubkey().is_err());
        assert!(w.pay(generate_random_hash(), 1).is_err());
        w.generate_keypair().unwrap();
        assert!(w.get_an_address().is_ok());
        assert!(w.get_a_pubkey().is_ok());
        assert!(w.pay(generate_random_hash(), 1).is_err());
    }

    #[test]
    pub fn rollback_at_fork() {
        let (mut w, pool, _ctx_update_source, hash) = new_wallet_pool_receiver_keyhash();
        receive(&mut w, &transaction_10_10(&hash));
        assert_eq!(w.balance().unwrap(), 100);
        // spend 100
        assert!(w.pay(generate_random_hash(), 100).is_ok());
        assert_eq!(w.balance().unwrap(), 0);
        let m = pool.lock().unwrap();
        let txs: Vec<Transaction> = m.get_transactions(1);
        drop(m);
        assert_eq!(txs.len(), 1);
        receive(&mut w, &txs[0]);
        assert_eq!(w.balance().unwrap(), 0);
        // rollback, after which we can spend 100 again!
        rollback(&mut w, &txs[0]);
        // after rollback, I can spend the 100 coins again!
        assert_eq!(w.balance().unwrap(), 100);
        assert!(w.pay(generate_random_hash(), 100).is_ok());
        assert_eq!(w.balance().unwrap(), 0);
    }
}
*/
