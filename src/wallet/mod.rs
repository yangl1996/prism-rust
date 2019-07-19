use crate::crypto::hash::Hashable;
use crate::transaction::{Address, Authorization, CoinId, Input, Output, Transaction};
use bincode::{deserialize, serialize};
use ed25519_dalek::{Keypair, Signature, KEYPAIR_LENGTH};
use rand::rngs::OsRng;
use rand::Rng;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};
use std::{error, fmt};
use crate::utxodb::{UtxoDatabase, Utxo, OutputWithTime};
use crate::block::pos_metadata::TimeStamp;

pub const COIN_CF: &str = "COIN";
pub const KEYPAIR_CF: &str = "KEYPAIR"; // &Address to &KeyPairPKCS8

pub type Result<T> = std::result::Result<T, WalletError>;

/// A data structure to maintain key pairs and their coins, and to generate transactions.
pub struct Wallet {
    /// The underlying RocksDB handle.
    db: rocksdb::DB,
    /// Keep key pair (in pkcs8 bytes) in memory for performance, it's duplicated in database as well.
    keypairs: Mutex<HashMap<Address, Keypair>>,
    /// Keep coin ids for mining in memory for performance
    // TODO: for now in create transaction, we don't touch these coins
    coins_mining: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    num_mining: RwLock<usize>,
    counter: AtomicUsize,
}

#[derive(Debug)]
pub enum WalletError {
    InsufficientBalance,
    MissingKeyPair,
    DBError(rocksdb::Error),
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WalletError::InsufficientBalance => write!(f, "insufficient balance"),
            WalletError::MissingKeyPair => write!(f, "missing key pair for the requested address"),
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
    fn open<P: AsRef<std::path::Path>>(path: P, mining_coins: usize) -> Result<Self> {
        let coin_cf = rocksdb::ColumnFamilyDescriptor::new(COIN_CF, rocksdb::Options::default());
        let keypair_cf =
            rocksdb::ColumnFamilyDescriptor::new(KEYPAIR_CF, rocksdb::Options::default());
        let mut db_opts = rocksdb::Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        let handle = rocksdb::DB::open_cf_descriptors(&db_opts, path, vec![coin_cf, keypair_cf])?;
        return Ok(Self {
            db: handle,
            keypairs: Mutex::new(HashMap::new()),
            coins_mining: Mutex::new(HashMap::new()),
            num_mining: RwLock::new(mining_coins),
            counter: AtomicUsize::new(0),
        });
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P, mining_coins: usize) -> Result<Self> {
        rocksdb::DB::destroy(&rocksdb::Options::default(), &path)?;
        return Self::open(path, mining_coins);
    }

    pub fn number_of_coins(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }

    /// Generate a new key pair
    pub fn generate_keypair(&self) -> Result<Address> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        let mut csprng: OsRng = OsRng::new().unwrap();
        let keypair: Keypair = Keypair::generate(&mut csprng);
        return self.load_keypair(keypair);
    }

    pub fn load_keypair(&self, keypair: Keypair) -> Result<Address> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        let addr: Address =
            ring::digest::digest(&ring::digest::SHA256, &keypair.public.as_bytes().as_ref()).into();
        self.db.put_cf(cf, &addr, &keypair.to_bytes().to_vec())?;
        let mut keypairs = self.keypairs.lock().unwrap();
        keypairs.insert(addr, keypair);
        Ok(addr)
    }

    /// Get the list of addresses for which we have a key pair
    pub fn addresses(&self) -> Result<Vec<Address>> {
        let keypairs = self.keypairs.lock().unwrap();
        let addrs = keypairs.keys().cloned().collect();
        Ok(addrs)
    }

    fn contains_keypair(&self, addr: &Address) -> bool {
        let keypairs = self.keypairs.lock().unwrap();
        if keypairs.contains_key(addr) {
            return true;
        }
        false
    }

    pub fn apply_diff(&self, add: &[Utxo], remove: &[Input]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        for utxo in add {
            // TODO: it's so funny that we have to do this for every added utxo
            if self.contains_keypair(&utxo.owner) {
                let output = Output {
                    value: utxo.value,
                    recipient: utxo.owner,
                };
                let output_with_time = OutputWithTime{
                    output: output,
                    confirm_time: utxo.confirm_time
                };
                let key = serialize(&utxo.coin).unwrap();
                let val = serialize(&output_with_time).unwrap();
                batch.put_cf(cf, &key, &val)?;
                let mut coins = self.coins_mining.lock().unwrap();
                if coins.len() < *self.num_mining.read().unwrap() {
                    coins.insert(key, val);
                }
                drop(coins);
                self.counter.fetch_add(1, Ordering::Relaxed);
            }
        }
        for coin in remove {
            let key = serialize(&coin.coin).unwrap();
            let mut coins = self.coins_mining.lock().unwrap();
            // no coins in coins_mining should be spent
            assert!(coins.remove(&key).is_none());
            drop(coins);
            batch.delete_cf(cf, &key)?;
        }
        self.db.write(batch)?;
        Ok(())
    }

    /// Returns the sum of values of all the coin in the wallet
    pub fn balance(&self) -> Result<u64> {
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        let balance = iter
            .map(|(_, v)| {
                let coin_data: OutputWithTime = bincode::deserialize(v.as_ref()).unwrap();
                coin_data.output.value
            })
            .sum::<u64>();
        Ok(balance)
    }

    /// Returns all the coins before a timestamp
    pub fn coins_before(&self, timestamp: TimeStamp) -> Result<Vec<(Utxo, [u8;KEYPAIR_LENGTH])>> {
        // let cf = self.db.cf_handle(COIN_CF).unwrap();
        // let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        let mut utxo_keypairs = vec![];
        let coins: Vec<(Vec<u8>,Vec<u8>)> = self.coins_mining.lock().unwrap().iter().map(|x|(x.0.clone(), x.1.clone())).collect();
        for (k,v) in coins {
            let coin_id: CoinId = bincode::deserialize(&k).unwrap();
            let coin_data: OutputWithTime = bincode::deserialize(&v).unwrap();
            if coin_data.confirm_time > timestamp {
                continue;
            }
            let keypairs = self.keypairs.lock().unwrap();
            if let Some(v) = keypairs.get(&coin_data.output.recipient) {
                let utxo = Utxo {
                    coin: coin_id,
                    value: coin_data.output.value,
                    owner: coin_data.output.recipient,
                    confirm_time: coin_data.confirm_time,
                };
                utxo_keypairs.push((utxo, v.to_bytes()));
            } else {
                return Err(WalletError::MissingKeyPair);
            }
            drop(keypairs);
        }
        Ok(utxo_keypairs)
    }

    /// Create a transaction using the wallet coins
    pub fn create_transaction(
        &self,
        recipient: Address,
        value: u64,
        previous_used_coin: Option<Input>,
    ) -> Result<Transaction> {
        let mut coins_to_use: Vec<Input> = vec![];
        let mut value_sum = 0u64;
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        let iter = match previous_used_coin {
            Some(c) => {
                let prev_key = serialize(&c.coin).unwrap();
                self.db.iterator_cf(
                    cf,
                    rocksdb::IteratorMode::From(&prev_key, rocksdb::Direction::Forward),
                )?
            }
            None => self.db.iterator_cf(cf, rocksdb::IteratorMode::Start)?,
        };
        // iterate through our wallet
        for (k, v) in iter {
            if self.coins_mining.lock().unwrap().contains_key(k.as_ref()) {
                // TODO: for now we don't use coins for mining, in case that we are running out of
                // coins for mining
                continue;
            }
            let coin_id: CoinId = bincode::deserialize(k.as_ref()).unwrap();
            let utxo_output: OutputWithTime = bincode::deserialize(v.as_ref()).unwrap();
            value_sum += utxo_output.output.value;
            coins_to_use.push(Input {
                coin: coin_id,
                value: utxo_output.output.value,
                owner: utxo_output.output.recipient,
            }); // coins that will be used for this transaction
            if value_sum >= value {
                // if we already have enough money, break
                break;
            }
        }
        if value_sum < value {
            // we don't have enough money in wallet
            return Err(WalletError::InsufficientBalance);
        }
        // if we have enough money in our wallet, create tx
        // remove used coin from wallet
        self.apply_diff(&vec![], &coins_to_use)?;

        // create the output
        let mut output = vec![Output { recipient, value }];
        if value_sum > value {
            // transfer the remaining value back to self
            let recipient = self.addresses()?[0];
            output.push(Output {
                recipient,
                value: value_sum - value,
            });
        }

        let mut owners: Vec<Address> = coins_to_use.iter().map(|input| input.owner).collect();
        let unsigned = Transaction {
            input: coins_to_use,
            output: output,
            authorization: vec![],
            hash: RefCell::new(None),
        };
        let mut authorization = vec![];
        owners.sort_unstable();
        owners.dedup();
        let raw_inputs = bincode::serialize(&unsigned.input).unwrap();
        let raw_outputs = bincode::serialize(&unsigned.output).unwrap();
        let raw_unsigned = [&raw_inputs[..], &raw_outputs[..]].concat();
        for owner in owners.iter() {
            let keypairs = self.keypairs.lock().unwrap();
            if let Some(v) = keypairs.get(&owner) {
                authorization.push(Authorization {
                    pubkey: v.public.to_bytes().to_vec(),
                    signature: v.sign(&raw_unsigned).to_bytes().to_vec(),
                });
            } else {
                return Err(WalletError::MissingKeyPair);
            }
            drop(keypairs);
        }
        self.counter
            .fetch_sub(unsigned.input.len(), Ordering::Relaxed);
        Ok(Transaction {
            authorization,
            ..unsigned
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::Wallet;
    use crate::crypto::hash::H256;
    use crate::transaction::tests::generate_random_coinid;
    use crate::transaction::{CoinId, Input};
    use crate::utxodb::Utxo;

    #[test]
    fn wallet() {
        let w = Wallet::new(std::path::Path::new("/tmp/walletdb_test.rocksdb")).unwrap();
        assert_eq!(w.balance().unwrap(), 0);
        assert_eq!(w.addresses().unwrap().len(), 0);
        let addr = w.generate_keypair().unwrap();
        assert_eq!(w.addresses().unwrap(), vec![addr]);
        assert!(w.create_transaction(H256::default(), 1, None).is_err());
        // give the test address 1000 x 10 coins
        let mut ico = vec![];
        let mut remove_ico = vec![];
        for _ in 0..1000 {
            let coin_id = generate_random_coinid();
            ico.push(Utxo {
                value: 10,
                owner: addr,
                coin: coin_id,
                confirm_time: 0,
            });
            remove_ico.push(Input {
                value: 10,
                owner: addr,
                coin: coin_id,
            });
        }
        w.apply_diff(&ico, &[]).unwrap();
        assert_eq!(w.balance().unwrap(), 10000);

        // generate transactions
        let tx = w.create_transaction(H256::default(), 19, None).unwrap();
        assert_eq!(tx.input.len(), 2);
        assert_eq!(tx.input[0].value, 10);
        assert_eq!(tx.input[1].value, 10);
        assert_eq!(tx.output.len(), 2);
        assert_eq!(tx.output[0].recipient, H256::default());
        assert_eq!(tx.output[0].value, 19);
        assert_eq!(tx.output[1].recipient, addr);
        assert_eq!(tx.output[1].value, 1);

        // remove coins
        w.apply_diff(&[], &remove_ico).unwrap();
        assert_eq!(w.balance().unwrap(), 0);
    }

}
