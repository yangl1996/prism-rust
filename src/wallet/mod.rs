use std::cell::RefCell;
use crate::crypto::hash::Hashable;
use crate::crypto::sign::{KeyPair, PubKey, Signable};
use crate::transaction::{Address, Authorization, CoinId, Input, Output, Transaction};
use bincode::{deserialize, serialize};
use std::{error, fmt};
use std::convert::TryInto;
use std::sync::Mutex;
use std::collections::HashMap;

pub const COIN_CF: &str = "COIN";
pub const KEYPAIR_CF: &str = "KEYPAIR";     // &Address to &KeyPairPKCS8

pub type Result<T> = std::result::Result<T, WalletError>;

/// A data structure to maintain key pairs and their coins, and to generate transactions.
pub struct Wallet {
    /// The underlying RocksDB handle.
    db: rocksdb::DB,
    /// Keep key pair (in pkcs8 bytes) in memory for performance, it's duplicated in database as well.
    key_pair: Mutex<HashMap<Address, Vec<u8>>>,
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
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let coin_cf =
            rocksdb::ColumnFamilyDescriptor::new(COIN_CF, rocksdb::Options::default());
        let keypair_cf =
            rocksdb::ColumnFamilyDescriptor::new(KEYPAIR_CF, rocksdb::Options::default());
        let mut db_opts = rocksdb::Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        let handle = rocksdb::DB::open_cf_descriptors(&db_opts, path, vec![coin_cf, keypair_cf])?;
        return Ok(Self { db: handle, key_pair: Mutex::new(HashMap::new()) });
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        rocksdb::DB::destroy(&rocksdb::Options::default(), &path)?;
        return Self::open(path);
    }

    /// Generate a new key pair
    pub fn generate_keypair(&self) -> Result<Address> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        let keypair = KeyPair::random();
        let k: Address = keypair.public_key().hash();
        let v = keypair.pkcs8_bytes;
        self.db.put_cf(cf, &k, &v)?;
        let mut key_pair = self.key_pair.lock().unwrap();
        key_pair.insert(k,v);
        Ok(k)
    }

    pub fn load_keypair(&self, keypair: KeyPair) -> Result<Address> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        let addr: Address = keypair.public_key().hash();
        self.db.put_cf(cf, &addr, &keypair.pkcs8_bytes)?;
        let mut key_pair = self.key_pair.lock().unwrap();
        key_pair.insert(addr,keypair.pkcs8_bytes);
        Ok(addr)
    }

    /// Get the list of addresses for which we have a key pair
    pub fn addresses(&self) -> Result<Vec<Address>> {
        let key_pair = self.key_pair.lock().unwrap();
        let addrs = key_pair.keys().cloned().collect();
        Ok(addrs)
    }

    fn keypair(&self, addr: &Address) -> Result<KeyPair> {
        let key_pair = self.key_pair.lock().unwrap();
        if let Some(v) = key_pair.get(addr) {
            return Ok(KeyPair::from_pkcs8(v.clone()));
        }
        Err(WalletError::MissingKeyPair)
    }

    fn contains_keypair(&self, addr: &Address) -> bool {
        let key_pair = self.key_pair.lock().unwrap();
        if key_pair.contains_key(addr) {
            return true;
        }
        false
    }

    pub fn apply_diff(&self, add: &[Input], remove: &[Input]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        for coin in add {
            // TODO: it's so funny that we have to do this for every added coin
            if self.contains_keypair(&coin.owner) {
                let output = Output {
                    value: coin.value,
                    recipient: coin.owner,
                };
                let key = serialize(&coin.coin).unwrap();
                let val = serialize(&output).unwrap();
                batch.put_cf(cf, &key, &val)?;
            }
        }
        for coin in remove {
            let key = serialize(&coin.coin).unwrap();
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
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        // iterate through our wallet
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

        let mut owners: Vec<Address> = coins_to_use.iter().map(|input|input.owner).collect();
        let unsigned = Transaction {
            input: coins_to_use,
            output: output,
            authorization: vec![],
            hash: RefCell::new(None),
        };
        let mut authorization = vec![];
        owners.sort_unstable();
        owners.dedup();
        for owner in owners.iter() {
            let keypair = self.keypair(&owner)?;
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

    /// Create multiple transactions at once
    pub fn create_transactions(&self, recipient_value: &[(Address, u64)]) -> Result<Vec<Transaction>> {
        let mut result: Vec<Transaction> = vec![];
        let mut coins_to_remove: Vec<Input> = vec![];
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        let mut iter = self.db.raw_iterator_cf(cf)?;
        iter.seek_to_first();
        for (recipient,value) in recipient_value {
            let mut coins_to_use: Vec<Input> = vec![];
            let mut value_sum = 0u64;
            // iterate through our wallet
            while iter.valid() {
                let coin_id: CoinId = bincode::deserialize(&iter.key().unwrap()).unwrap();
                let coin_data: Output = bincode::deserialize(&iter.value().unwrap()).unwrap();
                iter.next();
                value_sum += coin_data.value;
                coins_to_use.push(Input {
                    coin: coin_id,
                    value: coin_data.value,
                    owner: coin_data.recipient,
                }); // coins that will be used for this transaction
                if value_sum >= *value {
                    // if we already have enough money, break
                    break;
                }
            }
            if value_sum < *value {
                // we don't have enough money in wallet
                return Err(WalletError::InsufficientBalance);
            }
            // if we have enough money in our wallet, create tx
            // add coins_to_use to a vector and remove them at the end
            coins_to_remove.extend(&coins_to_use);
            // create the output
            let mut output = vec![Output { recipient: *recipient, value: *value }];
            if value_sum > *value {
                // transfer the remaining value back to self
                let recipient = self.addresses()?[0];
                output.push(Output {
                    recipient,
                    value: value_sum - *value,
                });
            }

            let mut owners: Vec<Address> = coins_to_use.iter().map(|input| input.owner).collect();
            let unsigned = Transaction {
                input: coins_to_use,
                output: output,
                authorization: vec![],
                hash: RefCell::new(None)
            };
            let mut authorization = vec![];
            owners.sort_unstable();
            owners.dedup();
            for owner in owners.iter() {
                let keypair = self.keypair(&owner)?;
                authorization.push(Authorization {
                    pubkey: keypair.public_key(),
                    signature: unsigned.sign(&keypair),
                });
            }

            result.push(Transaction {
                authorization,
                ..unsigned
            });
        }
        // remove used coin from wallet
        self.apply_diff(&[], &coins_to_remove)?;

        Ok(result)
    }
}


#[cfg(test)]
pub mod tests {
    use super::Wallet;
    use crate::transaction::{Input, CoinId};
    use crate::transaction::tests::generate_random_coinid;
    use crate::crypto::hash::H256;

    #[test]
    fn wallet() {
        let w = Wallet::new(std::path::Path::new("/tmp/walletdb_test.rocksdb")).unwrap();
        assert_eq!(w.balance().unwrap(), 0);
        assert_eq!(w.addresses().unwrap().len(), 0);
        let addr = w.generate_keypair().unwrap();
        assert_eq!(w.addresses().unwrap(), vec![addr]);
        assert!(w.create_transaction(H256::default(), 1).is_err());
        assert!(w.create_transactions(&[(H256::default(), 1)]).is_err());
        // give the test address 10 x 10 coins
        let mut ico: Vec<Input> = vec![];
        for _ in 0..10 {
            ico.push(
                Input{
                    value: 10,
                    owner: addr,
                    coin: generate_random_coinid(),
                });
        }
        w.apply_diff(&ico,&[]).unwrap();
        assert_eq!(w.balance().unwrap(), 100);

        // generate transactions
        let tx = w.create_transaction(H256::default(), 19).unwrap();
        assert_eq!(tx.input.len(),2);
        assert_eq!(tx.input[0].value,10);
        assert_eq!(tx.input[1].value,10);
        assert_eq!(tx.output.len(),2);
        assert_eq!(tx.output[0].recipient,H256::default());
        assert_eq!(tx.output[0].value,19);
        assert_eq!(tx.output[1].recipient,addr);
        assert_eq!(tx.output[1].value,1);

        let txs = w.create_transactions(&[(H256::default(), 8);4]).unwrap();
        assert_eq!(txs.len(),4);
        for i in 0..4usize {
            assert_eq!(txs[i].input.len(), 1);
            assert_eq!(txs[i].input[0].value, 10);
            assert_eq!(txs[i].output.len(), 2);
            assert_eq!(txs[i].output[0].recipient,H256::default());
            assert_eq!(txs[i].output[0].value, 8);
            assert_eq!(txs[i].output[1].recipient,addr);
            assert_eq!(txs[i].output[1].value, 2);
        }
        for i in 1..4usize {
            // these transactions would use different coins
            assert_ne!(txs[0].input[0].coin, txs[i].input[0].coin);
        }
        // remove coins
        w.apply_diff(&[],&ico).unwrap();
        assert_eq!(w.balance().unwrap(), 0);
    }

}
