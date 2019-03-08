pub mod validators;

use crate::crypto::hash::{Hashable, H256};

// TODO: ECDSA seems outdated. We should use EdDSA.

/// A Prism transaction. A transaction takes a set of existing coins and transforms them into a set
/// of output coins.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    input: Vec<Input>,
    output: Vec<Output>,
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        unimplemented!();
    }
}

/// An input of a transaction.
struct Input {
    /// A "pointer" to the coin being used.
    previous_output: OutPoint,
    /// The public key of the coin owner. We need this information because the transaction output
    /// only contains the hash of the public key, but the whole public key is needed to verify the
    /// signature.
    pubkey: PubKey,
    /// The signature by the coin owner. The coin owner signs this input using its private key to
    /// prove that it really owns this coin.
    signature: Signagure,
}

/// An output of a transaction.
// TODO: coinbase output (transaction fee). Maybe we don't need that in this case.
struct Output {
    /// The amount of this output.
    value: u64,
    /// The hash of the public key of the recipient (a.k.a. blockchain address).
    recipient: H256,
}

/// A "pointer" to a transaction output
struct OutPoint {
    /// The hash of the transaction being referred to.
    hash: H256,
    /// The index of the output in question in that transaction.
    index: u32,
}

