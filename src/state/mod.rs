use crate::transaction::{Input, Output};
use bincode::{deserialize, serialize};

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

#[derive(Debug)]
pub struct UTXODatabase {
    handle: rocksdb::DB,
}

impl UTXODatabase {
    pub fn new(path: &std::path::Path) -> Result<Self> {
        let db_handle = rocksdb::DB::open_default(path)?;
        return Ok(UTXODatabase {
            handle: db_handle,
        });
    }

    pub fn insert(&self, input: &Input, output: &Output) -> Result<()> {
        let input_serialized = serialize(input).unwrap();
        let output_serialized = serialize(output).unwrap();
        return self.handle.put(&input_serialized, &output_serialized);
    }

    pub fn delete(&mut self, input: &Input) -> Result<()> {
        let input_serialized = serialize(input).unwrap();
        return self.handle.delete(&input_serialized);
    }
}

// TODO: add tests
