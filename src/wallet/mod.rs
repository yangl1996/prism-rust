use crate::crypto::hash::Hashable;
use crate::transaction::{Address, Authorization, CoinId, Input, Output, Transaction};
use bincode::{deserialize, serialize};
use ed25519_dalek::{Keypair, Signature};
use rand::rngs::OsRng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::{error, fmt};
use rand::Rng;

pub const COIN_CF: &str = "COIN";
pub const KEYPAIR_CF: &str = "KEYPAIR"; // &Address to &KeyPairPKCS8

pub type Result<T> = std::result::Result<T, WalletError>;

/// A data structure to maintain key pairs and their coins, and to generate transactions.
pub struct Wallet {
    /// The underlying RocksDB handle.
    db: rocksdb::DB,
    /// Keep key pair (in pkcs8 bytes) in memory for performance, it's duplicated in database as well.
    keypairs: Mutex<HashMap<Address, Keypair>>,
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
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
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
            counter: AtomicUsize::new(0),
        });
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        rocksdb::DB::destroy(&rocksdb::Options::default(), &path)?;
        return Self::open(path);
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

    pub fn apply_diff(&self, add: &[(CoinId, Output)], remove: &[CoinId]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        for coin in add {
            if self.contains_keypair(&coin.1.recipient) {
                let key = serialize(&coin.0).unwrap();
                let val = serialize(&coin.1).unwrap();
                batch.put_cf(cf, &key, &val)?;
                self.counter.fetch_add(1, Ordering::Relaxed);
            }
        }
        for coin in remove {
            let key = serialize(&coin).unwrap();
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
    pub fn create_transaction(
        &self,
        recipient: Address,
        value: u64,
        previous_used_coin: Option<CoinId>,
    ) -> Result<Transaction> {
        let mut coins_to_use: Vec<CoinId> = vec![];
        let mut inputs: Vec<Input> = vec![];
        let mut value_sum = 0u64;
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        let previous_used_coin: Option<CoinId> = None;
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        let mut rng = rand::thread_rng();
        let val: u64 = rng.gen_range(0, 1000000000);
        // iterate through our wallet
        for (k, v) in iter {
            let coin_id: CoinId = bincode::deserialize(k.as_ref()).unwrap();
            let coin_data: Output = bincode::deserialize(v.as_ref()).unwrap();
            value_sum += val;
            coins_to_use.push(coin_id);
            inputs.push(Input {
                coin: coin_id,
                value: val,
                owner: coin_data.recipient,
            }); // coins that will be used for this transaction
            break;
        }
        // if we have enough money in our wallet, create tx
        // remove used coin from wallet
        //self.apply_diff(&vec![], &coins_to_use)?;

        // create the output
        let mut output = vec![Output { recipient, value:val }];

        let mut owners: Vec<Address> = inputs.iter().map(|input| input.owner).collect();
        let unsigned = Transaction {
            input: inputs,
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

    #[test]
    fn wallet() {
        let w = Wallet::new(std::path::Path::new("/tmp/walletdb_test.rocksdb")).unwrap();
        assert_eq!(w.balance().unwrap(), 0);
        assert_eq!(w.addresses().unwrap().len(), 0);
        let addr = w.generate_keypair().unwrap();
        assert_eq!(w.addresses().unwrap(), vec![addr]);
        assert!(w.create_transaction(H256::default(), 1, None).is_err());
        // give the test address 10 x 10 coins
        let mut ico: Vec<Input> = vec![];
        for _ in 0..10 {
            ico.push(Input {
                value: 10,
                owner: addr,
                coin: generate_random_coinid(),
            });
        }
        w.apply_diff(&ico, &[]).unwrap();
        assert_eq!(w.balance().unwrap(), 100);

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
        w.apply_diff(&[], &ico).unwrap();
        assert_eq!(w.balance().unwrap(), 0);
    }

}
