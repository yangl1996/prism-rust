use crate::crypto::hash::H256;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::transaction::{Address, CoinId, Output, Transaction};
use bincode::{deserialize, serialize};
use rocksdb::*;
use std::collections::HashSet;

pub struct UtxoDatabase {
    pub db: rocksdb::DB, // coin id to output
}

impl UtxoDatabase {
    /// Open the database at the given path, and create a new one if one is missing.
    fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let cfs = vec![];
        let mut opts = Options::default();
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(32));
        opts.set_allow_concurrent_memtable_write(false);
        let memtable_opts = MemtableFactory::HashSkipList {
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
        Ok(Self { db })
    }

    /// Create a new database at the given path, and initialize the content.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, rocksdb::Error> {
        DB::destroy(&Options::default(), &path)?;
        let db = Self::open(&path)?;

        Ok(db)
    }

    /// Check whether the given coin is in the UTXO set.
    pub fn contains(&self, coin: &CoinId) -> Result<bool, rocksdb::Error> {
        let result = self.db.get_pinned(serialize(&coin).unwrap())?;
        match result {
            Some(_) => return Ok(true),
            None => return Ok(false),
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
        Ok(checksum)
    }

    pub fn add_transaction(
        &self,
        t: &Transaction,
        hash: H256,
    ) -> Result<(Vec<(CoinId, Output)>, Vec<CoinId>), rocksdb::Error> {
        let mut added_coins: Vec<(CoinId, Output)> = vec![];
        let mut removed_coins: Vec<CoinId> = vec![];

        // use batch for the transaction
        let mut batch = rocksdb::WriteBatch::default();

        // check whether the inputs used in this transaction are all unspent, and whether the value
        // field in inputs are correct, and whether all owners have signed the transaction
        let mut owners: HashSet<Address> = HashSet::new();
        for input in &t.input {
            let id_ser = serialize(&input.coin).unwrap();
            match self.db.get_pinned(&id_ser)? {
                Some(d) => {
                    let coin_data: Output = deserialize(&d).unwrap();
                    owners.insert(coin_data.recipient);
                    if coin_data.value != input.value {
                        return Ok((vec![], vec![]));
                    }
                }
                None => return Ok((vec![], vec![])),
            }
            removed_coins.push(input.coin);
            batch.delete(&id_ser)?;
        }
        let signed_users: HashSet<Address> = t
            .authorization
            .iter()
            .map(|x| ring::digest::digest(&ring::digest::SHA256, &x.pubkey).into())
            .collect();
        if signed_users != owners {
            return Ok((vec![], vec![]));
        }

        // now that we have confirmed that all inputs are unspent, we will add the outputs and
        // commit to database
        for (idx, output) in t.output.iter().enumerate() {
            let id = CoinId {
                hash,
                index: idx as u32,
            };
            batch.put(serialize(&id).unwrap(), serialize(&output).unwrap())?;
            added_coins.push((id, *output));
        }
        // write the transaction as a batch
        // TODO: we don't write to wal here, so should the program crash, the db will be in
        // an inconsistent state. The solution here is to manually flush the memtable to
        // the disk at certain time, and manually log the state (e.g. voter tips, etc.)
        self.db.write_without_wal(batch)?;

        if !t.input.is_empty() {
            PERFORMANCE_COUNTER.record_confirm_transaction(&t);
        }

        Ok((added_coins, removed_coins))
    }

    pub fn remove_transaction(
        &self,
        t: &Transaction,
        hash: H256,
    ) -> Result<(Vec<(CoinId, Output)>, Vec<CoinId>), rocksdb::Error> {
        let mut added_coins: Vec<(CoinId, Output)> = vec![];
        let mut removed_coins: Vec<CoinId> = vec![];

        // use batch when committing
        let mut batch = rocksdb::WriteBatch::default();

        // check whether the outputs of this transaction are there. if so, this transaction was
        // valid when it was originally added
        for (idx, _out) in t.output.iter().enumerate() {
            let id = CoinId {
                hash,
                index: idx as u32,
            };
            let id_ser = serialize(&id).unwrap();
            if self.db.get_pinned(&id_ser)?.is_none() {
                return Ok((vec![], vec![]));
            }
            batch.delete(&id_ser)?;
            removed_coins.push(id);
        }

        // now that we have checked that this transaction was valid when originally added, we will
        // add back the input and commit to database
        for input in &t.input {
            let out = Output {
                value: input.value,
                recipient: input.owner,
            };
            batch.put(serialize(&input.coin).unwrap(), serialize(&out).unwrap())?;
            added_coins.push((input.coin, out));
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

        Ok((added_coins, removed_coins))
    }

    pub fn flush(&self) -> Result<(), rocksdb::Error> {
        let mut flush_opt = rocksdb::FlushOptions::default();
        flush_opt.set_wait(true);
        self.db.flush_opt(&flush_opt)
    }
}

#[cfg(test)]
mod test {}
