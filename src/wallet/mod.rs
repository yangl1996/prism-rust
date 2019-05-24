use crate::crypto::hash::Hashable;
use crate::crypto::sign::{KeyPair, PubKey, Signable};
use crate::transaction::{Address, Authorization, CoinId, Input, Output, Transaction};
use bincode::{deserialize, serialize};
use std::{error, fmt};
use std::convert::TryInto;

pub const COIN_CF: &str = "COIN";
pub const KEYPAIR_CF: &str = "KEYPAIR";     // &Address to &KeyPairPKCS8

pub type Result<T> = std::result::Result<T, WalletError>;

/// A data structure to maintain key pairs and their coins, and to generate transactions.
pub struct Wallet {
    /// The underlying RocksDB handle.
    db: rocksdb::DB,
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
        return Ok(Self { db: handle });
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
        Ok(k)
    }

    pub fn load_keypair(&self, keypair: KeyPair) -> Result<Address> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        let addr: Address = keypair.public_key().hash();
        self.db.put_cf(cf, &addr, &keypair.pkcs8_bytes)?;
        Ok(addr)
    }

    /// Get the list of addresses for which we have a key pair
    pub fn addresses(&self) -> Result<Vec<Address>> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        let mut iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start)?;
        let mut addrs = vec![];
        if let Some((k, _)) = iter.next() {
            let addr_bytes: [u8; 32] = (&k[0..32]).try_into().unwrap();
            let addr: Address = addr_bytes.into();
            addrs.push(addr);
        }
        Ok(addrs)
    }

    fn keypair(&self, addr: &Address) -> Result<KeyPair> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        if let Some(v) = self.db.get_cf(cf, &addr)? {
            let keypair = KeyPair::from_pkcs8(v.to_vec());
            return Ok(keypair);
        }
        Err(WalletError::MissingKeyPair)
    }

    fn contains_keypair(&self, addr: &Address) -> Result<bool> {
        let cf = self.db.cf_handle(KEYPAIR_CF).unwrap();
        if let Some(_) = self.db.get_cf(cf, &addr)? {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn apply_diff(&self, add: &[Input], remove: &[Input]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();
        let cf = self.db.cf_handle(COIN_CF).unwrap();
        for coin in remove {
            let key = serialize(&coin.coin).unwrap();
            batch.delete_cf(cf, &key)?;
        }
        for coin in add {
            // TODO: it's so funny that we have to do this for every added coin
            if self.contains_keypair(&coin.owner)? {
                let output = Output {
                    value: coin.value,
                    recipient: coin.owner,
                };
                let key = serialize(&coin.coin).unwrap();
                let val = serialize(&output).unwrap();
                batch.put_cf(cf, &key, &val)?;
            }
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

        // remove coins
        w.apply_diff(&[],&ico).unwrap();
        assert_eq!(w.balance().unwrap(), 0);
    }

}
