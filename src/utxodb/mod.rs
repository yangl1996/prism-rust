use crate::crypto::hash::Hashable;
use crate::transaction::{CoinId, Input, Output, Transaction};
use bincode::serialize;
use rocksdb::{Options, DB};

pub struct UtxoDatabase {
    db: rocksdb::DB, // coin id to output
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
            let num_outputs = t.output.len();

            // NOTE: we only undo the transaction if the outputs are there (it means that the
            // transaction was valid when we added it)
            let mut valid = true;
            for idx in 0..num_outputs {
                let id = CoinId {
                    hash: transaction_hash,
                    index: idx as u32,
                };
                if self.db.get(serialize(&id).unwrap())?.is_none() {
                    valid = false;
                }
            }

            if valid {
                // remove the outputs
                for idx in 0..num_outputs {
                    let id = CoinId {
                        hash: transaction_hash,
                        index: idx as u32,
                    };
                    self.db.delete(serialize(&id).unwrap())?;
                    let coin = Input {
                        coin: id,
                        value: t.output[idx].value,
                        owner: t.output[idx].recipient,
                    };
                    removed_coins.push(coin);
                }
                // add back the input
                for input in &t.input {
                    let out = Output {
                        value: input.value,
                        recipient: input.owner,
                    };
                    self.db
                        .put(serialize(&input.coin).unwrap(), serialize(&out).unwrap())?;
                    added_coins.push(input.clone());
                }
            }
        }

        // apply new transactions
        for t in added.iter() {
            // remove the input
            for input in &t.input {
                self.db.delete(serialize(&input.coin).unwrap())?;
                removed_coins.push(input.clone());
            }

            // add the output
            let transaction_hash = t.hash();
            for (idx, output) in t.output.iter().enumerate() {
                let id = CoinId {
                    hash: transaction_hash,
                    index: idx as u32,
                };
                self.db
                    .put(serialize(&id).unwrap(), serialize(&output).unwrap())?;
                let coin = Input {
                    coin: id,
                    value: output.value,
                    owner: output.recipient,
                };
                removed_coins.push(coin);
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
        };
        let transaction_2 = Transaction {
            input: vec![],
            output: vec![Output {
                value: 50,
                recipient: H256::default(),
            }],
            authorization: vec![],
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
