pub mod validators;
pub mod memory_pool;

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign;

/// A Prism transaction. A transaction takes a set of existing coins and transforms them into a set
/// of output coins.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    input: Vec<Input>,
    output: Vec<Output>,
    signatures: Vec<Signature>
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        unimplemented!();
    }
}

/// An input of a transaction.
#[derive(Serialize, Deserialize, Debug)]
struct Input {
    /// The hash of the transaction being referred to.
    hash: H256,
    /// The index of the output in question in that transaction.
    index: u32
}

/// An output of a transaction.
// TODO: coinbase output (transaction fee). Maybe we don't need that in this case.
#[derive(Serialize, Deserialize, Debug)]
struct Output {
    /// The amount of this output.
    value: u64,
    /// The hash of the public key of the recipient (a.k.a. blockchain address).
    recipient: H256,
}

#[derive(Serialize, Deserialize, Debug)]
struct Signature {
    pubkey: sign::PubKey,
    signature: sign::Signature,
}
