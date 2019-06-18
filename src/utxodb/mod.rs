use crate::crypto::hash::Hashable;
use crate::crypto::hash::H256;
use crate::transaction::{CoinId, Input, Output, Transaction};
use bincode::serialize;
use rocksdb::{Options, DB};
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;

pub struct UtxoDatabase {
    pub db: rocksdb::DB, // coin id to output
}

impl UtxoDatabase {
    /// Open the database at the given path, and create a new one if one is missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let cfs = vec![];
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

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
    pub fn contains(&self, coin: &CoinId) -> Result<bool, rocksdb::Error> {
        let result = self.db.get(serialize(&coin).unwrap())?;
        match result {
            Some(_) => return Ok(true),
            None => return Ok(false),
        };
    }

    pub fn snapshot(&self) -> Result<H256, rocksdb::Error> {
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
        for (k, v) in iter {
            ctx.update(k.as_ref());
            ctx.update(v.as_ref());
        }
        let hash = ctx.finish();
        return Ok(hash.into());
    }

    /// Remove the given transactions, then add another set of transactions.
    pub fn apply_diff(
        &self,
        added: &[Transaction],
        removed: &[Transaction],
    ) -> Result<(Vec<Input>, Vec<Input>), rocksdb::Error> {
        let mut added_coins: Vec<Input> = vec![];
        let mut removed_coins: Vec<Input> = vec![];
        // revert the transactions
        for t in removed.iter().rev() {
            // remove the output
            let transaction_hash = t.hash();

            // NOTE: we only undo the transaction if the outputs are there (it means that the
            // transaction was valid when we added it)
            let mut valid = true;
            let mut removed_coins_t: Vec<Input> = vec![];
            // use batch for the transaction
            let mut batch = rocksdb::WriteBatch::default();

            for (idx, out) in t.output.iter().enumerate() {
                let id = CoinId {
                    hash: transaction_hash,
                    index: idx as u32,
                };
                let id_ser = serialize(&id).unwrap();
                if self.db.get(&id_ser)?.is_none() {
                    valid = false;
                    break;
                }
                batch.delete(&id_ser)?;
                let coin = Input {
                    coin: id,
                    value: out.value,
                    owner: out.recipient,
                };
                removed_coins_t.push(coin);
            }

            if valid {
                // remove the outputs
                removed_coins.append(&mut removed_coins_t);

                // add back the input
                for input in &t.input {
                    let out = Output {
                        value: input.value,
                        recipient: input.owner,
                    };
                    batch
                        .put(serialize(&input.coin).unwrap(), serialize(&out).unwrap())?;
                    added_coins.push(input.clone());
                }
                //write the transaction as a batch
                self.db.write(batch)?;

                // TODO: it's a hack. The purpose is to ignore ICO transaction
                if !t.input.is_empty() {
                    PERFORMANCE_COUNTER.record_deconfirm_transaction(&t);
                }
            }
        }

        // apply new transactions
        for t in added.iter() {
            // NOTE: we only add the transaction if the inputs are valid and are unspent
            let mut valid = true;
            // use batch for the transaction
            let mut batch = rocksdb::WriteBatch::default();
            for input in &t.input {
                let id_ser = serialize(&input.coin).unwrap();
                if self.db.get(&id_ser)?.is_none() {
                    valid = false;
                    break;
                }
                batch.delete(&id_ser)?;
            }
            
            if valid {
                // remove the input
                removed_coins.extend(&t.input);

                // add the output
                let transaction_hash = t.hash();
                for (idx, output) in t.output.iter().enumerate() {
                    let id = CoinId {
                        hash: transaction_hash,
                        index: idx as u32,
                    };
                    batch
                        .put(serialize(&id).unwrap(), serialize(&output).unwrap())?;
                    let coin = Input {
                        coin: id,
                        value: output.value,
                        owner: output.recipient,
                    };
                    added_coins.push(coin);
                }
                //write the transaction as a batch
                self.db.write(batch)?;

                if !t.input.is_empty() {
                    PERFORMANCE_COUNTER.record_confirm_transaction(&t);
                }
            }
        }
        return Ok((added_coins, removed_coins));
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
        db.apply_diff(&vec![transaction_1.clone(), transaction_2.clone()], &vec![])
            .unwrap();
        let out: Output = deserialize(
            &db.db
                .get(
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
        db.apply_diff(&vec![transaction_3.clone()], &vec![])
            .unwrap();
        let out = db
            .db
            .get(
                serialize(&CoinId {
                    hash: transaction_1.hash(),
                    index: 0,
                })
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.is_none(), true);
        db.apply_diff(&vec![], &vec![transaction_2.clone(), transaction_3.clone()])
            .unwrap();
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
    }
}
