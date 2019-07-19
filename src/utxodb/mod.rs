use crate::crypto::hash::Hashable;
use crate::crypto::hash::H256;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::transaction::{CoinId, Input, Output, Transaction, Address};

use bincode::{serialize, deserialize};
use rocksdb::*;
use crate::block::pos_metadata::TimeStamp;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct OutputWithTime {
    pub output: Output,
    pub confirm_time: TimeStamp
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Utxo {
    pub coin: CoinId,
    pub value: u64,
    pub owner: Address,
    pub confirm_time: TimeStamp
}


const UNSPENT_COINS_CF: &str = "UNSPENT_COINS"; // Stores the current unspent coins -- the current state.
const SPENT_COINS_BUFFER_CF: &str = "SPENT_COINS_BUFFER"; // Stores the current spent coins in last Tau+tau_network_delay time.


pub struct UtxoDatabase {
    pub db: rocksdb::DB, // coin id to outputwithtime.
}

impl UtxoDatabase {
    /// Open the database at the given path, and create a new one if one is missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let unspent_coins_cf =
            ColumnFamilyDescriptor::new(UNSPENT_COINS_CF, Options::default());
        let spent_coins_buffer_cf =
            ColumnFamilyDescriptor::new(SPENT_COINS_BUFFER_CF, Options::default());
        let cfs = vec![unspent_coins_cf, spent_coins_buffer_cf];
        let mut opts = Options::default();
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(32));
        opts.set_allow_concurrent_memtable_write(false);
        let mut memtable_opts = MemtableFactory::HashSkipList {
            bucket_count: 1 << 20,
            height: 8,
            branching_factor: 4,
        };
        opts.set_memtable_factory(memtable_opts);
        // https://github.com/facebook/rocksdb/blob/671d15cbdd3839acb54cb21a2aa82efca4917155/options/options.cc#L509
        opts.optimize_for_point_lookup(512);
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.increase_parallelism(16);
        opts.set_max_background_flushes(2);
        opts.set_max_write_buffer_number(32);

        let db = DB::open_cf_descriptors(&opts, path, cfs)?;
        return Ok(Self { db: db });
    }

    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;

        return Ok(db);
    }

    /// Check whether the given coin is in the UTXO set.
    pub fn utxo_contains(&self, coin: &CoinId) -> Result<bool, rocksdb::Error> {
        let unspent_coins_cf = self.db.cf_handle(UNSPENT_COINS_CF).unwrap();
        let result = self.db.get_pinned_cf(unspent_coins_cf,serialize(&coin).unwrap())?;
        match result {
            Some(_) => return Ok(true),
            None => return Ok(false),
        };
    }

    /// Check whether the given coin is unspent at time 'timestamp'.
    pub fn is_coin_unspent(&self, coin: &CoinId, timestamp: TimeStamp)-> Result<bool, rocksdb::Error> {
        let unspent_coins_cf = self.db.cf_handle(UNSPENT_COINS_CF).unwrap();
        let spent_coins_buffer_cf = self.db.cf_handle(SPENT_COINS_BUFFER_CF).unwrap();
        // first check if the coin is present in unspent_coins cf
        let option_output_with_time = self.db.get_cf(unspent_coins_cf,serialize(&coin).unwrap())?;
        match option_output_with_time {
            Some(ser_output_with_time) => {
                let output_with_time: OutputWithTime = deserialize(&ser_output_with_time).unwrap();
                if output_with_time.confirm_time < timestamp {
                    return Ok(true);
                }
            },
            None => {},
        };

        // second check if the coin is present in spent_coins_buffer cf
        let option_output_with_time = self.db.get_cf(spent_coins_buffer_cf,serialize(&coin).unwrap())?;
        match option_output_with_time {
            Some(ser_output_with_time) => {
                let output_with_time: OutputWithTime = deserialize(&ser_output_with_time).unwrap();
                if output_with_time.confirm_time < timestamp {
                    return Ok(true);
                }
                else{
                    return Ok(false);
                }
            },
            None => {return Ok(false)},
        };
    }

    pub fn snapshot(&self) -> Result<Vec<u8>, rocksdb::Error> {
        let mut iter_opt = rocksdb::ReadOptions::default();
        iter_opt.set_prefix_same_as_start(false);
        iter_opt.set_total_order_seek(true);
        let iter = self
            .db
            .iterator_opt(rocksdb::IteratorMode::Start, &iter_opt);
        let mut inited = false;
        let mut checksum: Vec<u8> = vec![];
        for (k, _) in iter {
            if !inited {
                checksum = vec![0; k.as_ref().len()];
                inited = true;
            }
            checksum = checksum
                .iter()
                .zip(k.as_ref())
                .map(|(&c, &k)| c ^ k)
                .collect();
        }
        return Ok(checksum);
    }

    pub fn add_transaction(
        &self,
        t: &Transaction,
        hash: H256,
        tx_confirm_time: TimeStamp
    ) -> Result<(Vec<Utxo>, Vec<Input>), rocksdb::Error> {
        let unspent_coins_cf = self.db.cf_handle(UNSPENT_COINS_CF).unwrap();
        let spent_coins_buffer_cf = self.db.cf_handle(SPENT_COINS_BUFFER_CF).unwrap();

        let mut added_utxos: Vec<Utxo> = vec![];
        let mut removed_inputs: Vec<Input> = vec![];

        // use batch for the transaction
        let mut batch = rocksdb::WriteBatch::default();

        // check whether the inputs used in this transaction are all unspent
        for input in &t.input {
            let id_ser = serialize(&input.coin).unwrap();
            let option_output_with_time = self.db.get_cf(unspent_coins_cf, &id_ser)?;
            match option_output_with_time {
                Some(ser_output_with_time) => {
                    // TODO: Can we move data from one cf to another?
                    // Removing the coin from unspent cf to spent cf
                    batch.delete_cf(unspent_coins_cf, &id_ser)?;
                    batch.put_cf(spent_coins_buffer_cf, &id_ser, &ser_output_with_time)?;
                }
                None => {
                    return Ok((vec![], vec![]));
                }
            }
        }

        // remove the input
        removed_inputs = t.input.clone();

        // now that we have confirmed that all inputs are unspent, we will add the outputs and
        // commit to unspent_coins_cf of the database
        for (idx, output) in t.output.iter().enumerate() {
            let id = CoinId {
                hash: hash,
                index: idx as u32,
            };
            let output_with_time = OutputWithTime {
                output: *output,
                confirm_time: tx_confirm_time,
            };
            batch.put_cf(unspent_coins_cf, serialize(&id).unwrap(), serialize(&output_with_time).unwrap())?;
            let utxo = Utxo {
                coin: id,
                value: output.value,
                owner: output.recipient,
                confirm_time: tx_confirm_time
            };
            added_utxos.push(utxo);
        }
        // write the transaction as a batch
        // TODO: we don't write to wal here, so should the program crash, the db will be in
        // an inconsistent state. The solution here is to manually flush the memtable to
        // the disk at certain time, and manually log the state (e.g. voter tips, etc.)
        self.db.write_without_wal(batch)?;

        if !t.input.is_empty() {
            PERFORMANCE_COUNTER.record_confirm_transaction(&t);
        }

        return Ok((added_utxos, removed_inputs));
    }

    pub fn remove_transaction(
        &self,
        t: &Transaction,
        hash: H256,
        timestamp: TimeStamp
    ) -> Result<(Vec<Utxo>, Vec<Input>), rocksdb::Error> {
        let unspent_coins_cf = self.db.cf_handle(UNSPENT_COINS_CF).unwrap();
        let spent_coins_buffer_cf = self.db.cf_handle(SPENT_COINS_BUFFER_CF).unwrap();
        let mut removed_coins: Vec<Input> = vec![];
        let mut added_utxos: Vec<Utxo> = vec![];

        // use batch when committing
        let mut batch = rocksdb::WriteBatch::default();

        // check whether the outputs of this transaction are there. if so, this transaction was
        // valid when it was originally added
        for (idx, out) in t.output.iter().enumerate() {
            let id = CoinId {
                hash: hash,
                index: idx as u32,
            };
            let id_ser = serialize(&id).unwrap();
            if self.db.get_pinned_cf(unspent_coins_cf,&id_ser)?.is_none() {
                unreachable!();
            }
            batch.delete_cf(unspent_coins_cf, &id_ser)?;

            // reconstruct the output coin that is being deleted
            let coin = Input {
                coin: id,
                value: out.value,
                owner: out.recipient,
            };
            removed_coins.push(coin);
        }

        // now that we have checked that this transaction was valid when originally added, we will
        // add back the input and commit to database
        for input in &t.input {
            let id_ser = serialize(&input.coin).unwrap();
            let option_output_with_time = self.db.get_cf(spent_coins_buffer_cf, &id_ser)?;
            match option_output_with_time {
                Some(ser_output_with_time) => {
                    // TODO: Can we move data from one cf to another?
                    // Removing the coin from spent cf to unspent cf
                    batch.delete_cf(spent_coins_buffer_cf, &id_ser)?;
                    batch.put_cf(unspent_coins_cf, &id_ser, &ser_output_with_time)?;
                    let output_with_time: OutputWithTime = deserialize(&ser_output_with_time).unwrap();
                    let utxo = Utxo{
                        coin: input.coin,
                        value: output_with_time.output.value,
                        owner: output_with_time.output.recipient,
                        confirm_time: output_with_time.confirm_time,
                    };
                    added_utxos.push(utxo);
                }
                None => {
                    unreachable!();
                }
            }
            // TODO: The timestamp of the original coin is unknown at this point.
            // panic!("UTXO remove_transactions() should be be called because the timestamp issue is not yet fixed");
        }
        // write the transaction as a batch
        // TODO: we don't write to wal here, so should the program crash, the db will be in
        // an inconsistent state. The solution here is to manually flush the memtable to
        // the disk at certain time, and manually log the state (e.g. voter tips, etc.)
        self.db.write_without_wal(batch)?;

        // TODO: it's a hack. The purpose is to ignore ICO transaction
        if !t.input.is_empty() {
            PERFORMANCE_COUNTER.record_deconfirm_transaction(&t);
        }

        return Ok((added_utxos, removed_coins));
    }

    /// Delete all the coins which were spent before timestamp
    pub fn delete_old_spent_coins(&self, timestamp: TimeStamp) {
        let timestamp = timestamp;
        let spent_coins_buffer_cf = self.db.cf_handle(SPENT_COINS_BUFFER_CF).unwrap();
        let iter = self.db.iterator_cf(spent_coins_buffer_cf, rocksdb::IteratorMode::Start).unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        for (k,v) in iter {
            let output_with_time: OutputWithTime = deserialize(v.as_ref()).unwrap();
            if output_with_time.confirm_time < timestamp {
                batch.delete_cf(spent_coins_buffer_cf, k);
            }
        }
    }

    pub fn flush(&self) -> Result<(), rocksdb::Error> {
        let mut flush_opt = rocksdb::FlushOptions::default();
        flush_opt.set_wait(true);
        self.db.flush_opt(&flush_opt)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::hash::H256;
    use crate::transaction::Input;
    use bincode::deserialize;
    use std::cell::RefCell;

    #[test]
    fn initialize_new() {
        let _db = UtxoDatabase::new("/tmp/prism_test_ledger_new.rocksdb").unwrap();
    }

    #[test]
    fn apply_diff() {
        let db = UtxoDatabase::new("/tmp/prism_test_ledger_apply_diff.rocksdb").unwrap();

        let unspent_coins_cf = db.db.cf_handle(UNSPENT_COINS_CF).unwrap();
        let spent_coins_buffer_cf = db.db.cf_handle(SPENT_COINS_BUFFER_CF).unwrap();
        let transaction_1 = Transaction {
            input: vec![],
            output: vec![
                Output {
                    value: 100,
                    recipient: H256::default(),
                },
                Output {
                    value: 75,
                    recipient: H256::default(),
                },
            ],
            authorization: vec![],
            hash: RefCell::new(None),
        };
        let transaction_2 = Transaction {
            input: vec![],
            output: vec![Output {
                value: 50,
                recipient: H256::default(),
            }],
            authorization: vec![],
            hash: RefCell::new(None),
        };
        db.add_transaction(&transaction_1, transaction_1.hash(), 0).unwrap();
        db.add_transaction(&transaction_2, transaction_2.hash(), 0).unwrap();
        let out: Output = deserialize(
            &db.db
                .get_cf(unspent_coins_cf,
                    serialize(&CoinId {
                        hash: transaction_1.hash(),
                        index: 1,
                    })
                    .unwrap(),
                )
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            out,
            Output {
                value: 75,
                recipient: H256::default(),
            }
        );
        let transaction_3 = Transaction {
            input: vec![Input {
                coin: CoinId {
                    hash: transaction_1.hash(),
                    index: 0,
                },
                value: 100,
                owner: H256::default(),
            }],
            output: vec![Output {
                value: 100,
                recipient: H256::default(),
            }],
            authorization: vec![],
            hash: RefCell::new(None),
        };
        db.add_transaction(&transaction_3, transaction_3.hash(), 0).unwrap();
        let out = db
            .db
            .get_cf(unspent_coins_cf,
                serialize(&CoinId {
                    hash: transaction_1.hash(),
                    index: 0,
                })
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.is_none(), true);
        /*
        db.remove_transaction(&transaction_2, transaction_2.hash(), 0).unwrap();
        db.remove_transaction(&transaction_3, transaction_3.hash(), 0).unwrap();
        let out: Output = deserialize(
            &db.db
                .get(
                    serialize(&CoinId {
                        hash: transaction_1.hash(),
                        index: 0,
                    })
                    .unwrap(),
                )
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            out,
            Output {
                value: 100,
                recipient: H256::default(),
            }
        );
        let out = db
            .db
            .get(
                serialize(&CoinId {
                    hash: transaction_2.hash(),
                    index: 0,
                })
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.is_none(), true);
        */
    }
}
