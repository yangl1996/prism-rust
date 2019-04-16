pub mod generator;

use crate::block::transaction::Content as TxContent;
use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Input, Output, Transaction};

use bincode::{deserialize, serialize};
use std::sync::Mutex;

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

/// The struct that identifies an UTXO, it contains two fields of Input
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoinId {
    /// The hash of the transaction being referred to.
    pub hash: H256,
    /// The index of the output in question in that transaction.
    pub index: u32,
}

impl From<&Input> for CoinId {
    fn from(other: &Input) -> Self {
        Self { hash: other.hash, index: other.index}
    }
}

pub type CoinData = Output;

// Bitcoin UTXO is much more complicated because they have extra seg-wit and locktime.
pub struct UTXO {
    pub coin_id: CoinId, // Hash of the transaction. This along with the index is the coin index is the key.
    pub coin_data: CoinData,
}

pub struct UTXODatabase {
    handle: rocksdb::DB,
    count: Mutex<u64>,
}

impl UTXODatabase {
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let db_handle = rocksdb::DB::open_default(path)?;
        return Ok(UTXODatabase {
            handle: db_handle,
            count: Mutex::new(0),
        });
    }

    pub fn insert(&self, utxo: &UTXO) -> Result<()> {
        let key = serialize(&utxo.coin_id).unwrap();
        let value = serialize(&utxo.coin_data).unwrap();
        let mut count = self.count.lock().unwrap();
        *count += 1;
        return self.handle.put(&key, &value);
    }

    pub fn delete(&mut self, coin_id: &CoinId) -> Result<()> {
        let key = serialize(coin_id).unwrap();
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        return self.handle.delete(key);
    }

    pub fn get(&self, coin_id: &CoinId) -> Result<Option<CoinData>> {
        let key = serialize(coin_id).unwrap();
        let serialized = self.handle.get(&key)?;
        match serialized {
            None => return Ok(None),
            Some(s) => return Ok(Some(deserialize(&s).unwrap())),
        }
    }

    //TODO: Check the key without getting the value (Use Bloom filters maybe?)
    pub fn check(&mut self, coin_id: &CoinId) -> Result<bool> {
        let key = serialize(coin_id).unwrap();
        let serialized = self.handle.get(&key)?;
        match serialized {
            None => return Ok(false),
            Some(s) => return Ok(true),
        }
    }

    pub fn num_utxo(&self) -> u64 {
        let count = self.count.lock().unwrap();
        return *count;
    }

    /// Delete the spent coins, then add coins in a transaction
    pub fn receive(&mut self, tx: &Transaction) -> Result<()> {
        let hash: H256 = tx.hash(); // compute hash here, and below inside Input we don't have to compute again (we just copy)
        for input in tx.input.iter() {
            let coin_id: CoinId = input.into();
            self.delete(&coin_id)?;
        }
        for (index, output) in tx.output.iter().enumerate() {
            let utxo = UTXO {
                coin_id: CoinId {
                    hash,
                    index: index as u32,
                },
                coin_data: CoinData {
                    value: output.value,
                    recipient: output.recipient,
                },
            };
            self.insert(&utxo)?;
        }
        Ok(())
    }

    pub fn rollback(&mut self, tx: &Transaction) -> Result<()> {
        let hash: H256 = tx.hash();
        for index in 0..tx.output.len() {
            let coin_id = CoinId {
                hash,
                index: index as u32,
            };
            self.delete(&coin_id)?;
        }
        for (index, input) in tx.input.iter().enumerate() {
            // Get the value
            let utxo = UTXO {
                coin_id: input.into(),
                coin_data: CoinData {
                    value: input.value,
                    recipient: input.recipient,
                },
            };
            self.insert(&utxo)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::{generator, CoinData, CoinId, UTXODatabase, UTXO};
    use crate::crypto::generator as crypto_generator;
    use crate::crypto::hash::{Hashable, H256};
    use crate::transaction::{generator as tx_generator, Transaction};

    fn init_with_tx(state_db: &mut UTXODatabase, tx: &Transaction) {
        let hash: H256 = tx.hash(); // compute hash here, and below inside Input we don't have to compute again (we just copy)
        for input in tx.input.iter() {
            let coin_id = CoinId {
                hash: input.hash,
                index: input.index,
            };
            let coin_data = CoinData {
                value: 1,
                recipient: crypto_generator::h256(),
            };
            let utxo = UTXO { coin_id, coin_data };
            if state_db.insert(&utxo).is_err() {
                panic!("State DB error.");
            }
        }
    }

    //    #[test]
    //    fn insert_get_check_and_delete() {
    //        let mut state_db = generator::random();
    //        let mut count = state_db.num_utxo();
    //
    //        println!("Test 1: count");
    //        let transaction = tx_generator::random();
    //        let utxos = generator::tx_to_utxos(transaction);
    //        for utxo in utxos.iter() {
    //            state_db.insert(utxo);
    //        }
    //
    //        assert_eq!(state_db.num_utxo(), count + utxos.len() as u64);
    //
    //        println!("Test 2: check()");
    //        for utxo in utxos.iter() {
    //            assert!(state_db.check(&utxo.coin_id).unwrap());
    //        }
    //
    //        println!("Test 3: get()");
    //        for utxo in utxos.iter() {
    //            assert_eq!(state_db.get(&utxo.coin_id).unwrap().unwrap(), utxo.value);
    //        }
    //
    //        println!("Test 4: delete()");
    //        state_db.delete(&utxos[0].coin_id);
    //        assert!(!state_db.check(&utxos[0].coin_id).unwrap());
    //
    //        assert_eq!(state_db.num_utxo(), count + utxos.len() as u64 - 1);
    //    }

    #[test]
    pub fn create_receive() {
        let state_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
        let mut state_db = UTXODatabase::new(state_path).unwrap();
        let tx = tx_generator::random();
        init_with_tx(&mut state_db, &tx);
        assert_eq!(state_db.num_utxo() as usize, tx.input.len());
        assert!(state_db.receive(&tx).is_ok());
        assert_eq!(state_db.num_utxo() as usize, tx.output.len());
        let hash = tx.hash();
        for index in 0..tx.output.len() as u32 {
            assert_eq!(state_db.check(&CoinId { hash, index }), Ok(true));
            let coin_data = state_db.get(&CoinId { hash, index }).unwrap().unwrap();
            assert_eq!(coin_data, tx.output[index as usize])
        }
        drop(state_db);
        assert!(rocksdb::DB::destroy(
            &rocksdb::Options::default(),
            "/tmp/prism_test_state.rocksdb"
        )
        .is_ok());
    }
}

// TODO: add tests
