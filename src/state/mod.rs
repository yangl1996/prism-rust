pub mod generator;
pub mod updater;


use crate::crypto::hash::{Hashable, H256};
use crate::transaction::{Input, Output};

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
        Self {
            hash: other.hash,
            index: other.index,
        }
    }
}

pub type CoinData = Output;

// Bitcoin UTXO is much more complicated because they have extra seg-wit and locktime.
pub struct UTXO {
    pub coin_id: CoinId,
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

    pub fn delete(&self, coin_id: &CoinId) -> Result<()> {
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
    pub fn check(&self, coin_id: &CoinId) -> Result<bool> {
        let key = serialize(coin_id).unwrap();
        let serialized = self.handle.get(&key)?;
        match serialized {
            None => return Ok(false),
            Some(_s) => return Ok(true),
        }
    }

    pub fn num_utxo(&self) -> u64 {
        let count = self.count.lock().unwrap();
        return *count;
    }

    /// Update the state.
    /// Can serve as add or rollback, based on arguments to_delete and to_insert.
    pub fn update(&self, to_delete: &Vec<CoinId>, to_insert: &Vec<UTXO>) -> Result<()> {
        for coin_id in to_delete {
            self.delete(coin_id)?;
        }
        for utxo in to_insert {
            self.insert(utxo)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::{generator, CoinData, CoinId, UTXODatabase, UTXO};
    use crate::crypto::generator as crypto_generator;
    use crate::crypto::hash::{Hashable, H256};
    use crate::handler::{to_coinid_and_potential_utxo, to_rollback_coinid_and_potential_utxo};
    use crate::transaction::{generator as tx_generator, Input, Transaction};

    fn init_with_tx_input(state_db: &mut UTXODatabase, tx: &Transaction) {
        let _hash: H256 = tx.hash(); // compute hash here, and below inside Input we don't have to compute again (we just copy)
        for input in tx.input.iter() {
            let coin_id: CoinId = input.into();
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

    fn try_receive_transaction(state_db: &mut UTXODatabase, tx: &Transaction) {
        let (to_delete, to_insert) = to_coinid_and_potential_utxo(tx);
        assert!(state_db.update(&to_delete, &to_insert).is_ok());
        // assume this tx spends all utxo in state
        assert_eq!(state_db.num_utxo() as usize, tx.output.len());
        let hash = tx.hash();
        for index in 0..tx.output.len() {
            assert_eq!(
                state_db.check(&CoinId {
                    hash,
                    index: index as u32
                }),
                Ok(true)
            );
            let coin_data = state_db
                .get(&CoinId {
                    hash,
                    index: index as u32,
                })
                .unwrap()
                .unwrap();
            assert_eq!(coin_data, tx.output[index])
        }
    }
    fn try_rollback_transaction(state_db: &mut UTXODatabase, tx: &Transaction) {
        let (to_delete, to_insert) = to_rollback_coinid_and_potential_utxo(tx);
        assert!(state_db.update(&to_delete, &to_insert).is_ok());
    }
    #[test]
    pub fn create_receive_rollback() {
        let mut state_db = generator::random();
        let tx = tx_generator::random();
        // we have to init with the inputs, otherwise we cannot receive a tx
        init_with_tx_input(&mut state_db, &tx);
        assert_eq!(state_db.num_utxo() as usize, tx.input.len());
        // receive tx
        try_receive_transaction(&mut state_db, &tx);
        // rollback tx, after rollback, the db should be identical just after init_with_tx_input
        try_rollback_transaction(&mut state_db, &tx);
        assert_eq!(state_db.num_utxo() as usize, tx.input.len());
        drop(state_db);
    }

    #[test]
    pub fn rollback_at_fork() {
        let mut state_db = generator::random();
        let tx0 = tx_generator::random();
        let input1: Vec<Input> = (0..tx0.output.len())
            .map(|i| Input {
                hash: tx0.hash(),
                index: i as u32,
                value: tx0.output[i].value,
                recipient: tx0.output[i].recipient,
            })
            .collect();
        let tx1 = Transaction {
            input: input1.clone(),
            ..tx_generator::random()
        };
        let tx2 = Transaction {
            input: input1.clone(),
            ..tx_generator::random()
        };
        /*
        tx0 <---- tx1
              |
              --- tx2
        */
        // we have to init with the inputs, otherwise we cannot receive a tx
        init_with_tx_input(&mut state_db, &tx0);
        assert_eq!(state_db.num_utxo() as usize, tx0.input.len());
        // receive tx0
        try_receive_transaction(&mut state_db, &tx0);
        // receive tx1
        try_receive_transaction(&mut state_db, &tx1);

        // rollback tx1, after rollback, the db should be identical just after receive tx0
        try_rollback_transaction(&mut state_db, &tx1);
        assert_eq!(state_db.num_utxo() as usize, tx0.output.len());
        // receive tx2
        try_receive_transaction(&mut state_db, &tx2);
        drop(state_db);
        assert!(rocksdb::DB::destroy(
            &rocksdb::Options::default(),
            "/tmp/prism_test_state_2.rocksdb"
        )
        .is_ok());
    }
}
