pub mod generator;

use crate::crypto::hash::{Hashable, H256};
use crate::crypto::sign;
use bincode::serialize;
use crate::crypto::sign::{Signable, PubKey, KeyPair, Signature};

/// A Prism transaction. A transaction takes a set of existing coins and transforms them into a set
/// of output coins.
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub input: Vec<Input>,
    pub output: Vec<Output>,
    pub key_sig: Vec<KeyAndSignature>,
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        return ring::digest::digest(&ring::digest::SHA256, &serialize(self).unwrap()).into();
    }
}

impl Signable for Transaction {
    fn sign(&self, keypair: &KeyPair) -> Signature {
        //only sign fields input, output. not signatures.
        let unsigned_input = serialize(&self.input).unwrap();
        let unsigned_output = serialize(&self.output).unwrap();
        let unsigned = [&unsigned_input[..], &unsigned_output[..]].concat();// we can also use Vec extend, don't know which is better
        keypair.sign(&unsigned)
    }

    fn verify(&self, public_key: &PubKey, signature: &Signature) -> bool {
        let unsigned_input = serialize(&self.input).unwrap();
        let unsigned_output = serialize(&self.output).unwrap();
        let unsigned = [&unsigned_input[..], &unsigned_output[..]].concat();// we can also use Vec extend, don't know which is better
        public_key.verify(&unsigned, signature)
    }

}

/// An input of a transaction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    /// The hash of the transaction being referred to.
    pub hash: H256,
    /// The index of the output in question in that transaction.
    pub index: u32,
    /// The amount of this input, this is redundant since it is also stored in the transaction referred by hash
    pub value: u64,
    /// The hash of the public key of the recipient of this input, this is redundant since it is also stored in the transaction referred by hash
    pub recipient: H256,
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
pub struct KeyAndSignature {
    pub public_key: PubKey,
    pub signature: Signature,
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{0:<10} | {1:<10}]", self.hash, self.index)?;
        Ok(())
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{0:<2} | {1:<10}]", self.value, self.recipient)?;
        Ok(())
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Tx: ")?;
        for input in self.input.iter() {
            write!(f, "{} ", input)?;
        }

        write!(f, " ==> ")?;
        for output in self.output.iter() {
            write!(f, "{} ", output)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn sign() {
        let unsigned = generator::random();
        let keypair = KeyPair::new();
        let signature = unsigned.sign(&keypair);
        assert!(unsigned.verify(&keypair.public_key(), &signature));
        let keypair_2 = KeyPair::new();
        assert!(!unsigned.verify(&keypair_2.public_key(), &signature));
    }
}