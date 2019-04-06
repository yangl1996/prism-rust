pub mod generator;

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign;
use bincode::serialize;

/// A Prism transaction. A transaction takes a set of existing coins and transforms them into a set
/// of output coins.
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub input: Vec<Input>,
    pub output: Vec<Output>,
    pub signatures: Vec<Signature>,
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        return ring::digest::digest(&ring::digest::SHA256, &serialize(self).unwrap()).into();
    }
}

/// An input of a transaction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    /// The hash of the transaction being referred to.
    pub hash: H256,
    /// The index of the output in question in that transaction.
    pub index: u32,
}

/// An output of a transaction.
// TODO: coinbase output (transaction fee). Maybe we don't need that in this case.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Output {
    /// The amount of this output.
    pub value: u64,
    /// The hash of the public key of the recipient (a.k.a. blockchain address).
    pub recipient: H256,
}

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Signature {
    pub pubkey: sign::PubKey,
    pub signature: sign::Signature,
}
