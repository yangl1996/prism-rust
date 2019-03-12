pub mod validators;
pub mod memory_pool;
pub mod fee;

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign;
use bincode::{serialize, deserialize};
use std::{fmt, cmp};

/// A Prism transaction. A transaction takes a set of existing coins and transforms them into a set
/// of output coins.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    signatures: Vec<Signature>
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        //don't know if this is right?
        return ring::digest::digest(&ring::digest::SHA256, &serialize(self).unwrap()[..]).into();
    }
}

/// An input of a transaction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// The hash of the transaction being referred to.
    hash: H256,
    /// The index of the output in question in that transaction.
    index: u32
}

/// An output of a transaction.
// TODO: coinbase output (transaction fee). Maybe we don't need that in this case.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The amount of this output.
    value: u64,
    /// The hash of the public key of the recipient (a.k.a. blockchain address).
    recipient: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Signature {
    pubkey: sign::PubKey,
    signature: sign::Signature,
}

#[derive(Default, Clone)]
pub struct IndexedTransaction {
    pub hash: H256,
    pub raw: Transaction,
}

impl From<&'static str> for Transaction {
    fn from(s: &'static str) -> Self {
        unimplemented!()
        //deserialize(&s.from_hex::<Vec<u8>>().unwrap() as &[u8]).unwrap()
    }
}

impl<T> From<T> for IndexedTransaction where Transaction: From<T> {
    fn from(other: T) -> Self {
        let tx = Transaction::from(other);
        IndexedTransaction {
            hash: tx.hash(),
            raw: tx,
        }
    }
}

impl IndexedTransaction {
    pub fn new(hash: H256, transaction: Transaction) -> Self {
        IndexedTransaction {
            hash: hash,
            raw: transaction,
        }
    }
}

impl cmp::PartialEq for IndexedTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}