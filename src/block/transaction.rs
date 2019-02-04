extern crate bincode;
extern crate ring;

use super::address;
use super::hash;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    source: address::Address,
    destination: address::Address,
    amount: u32,
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{\n")?;
        write!(f, "  source: {}\n", self.source)?;
        write!(f, "  destination: {}\n", self.destination)?;
        write!(f, "  amount: {}\n", self.amount)?;
        write!(f, "}}")
    }
}

// hashing a single Txn makes little sense, since we include the hash of all
// transaction data in the block header
impl hash::Hashable for [Transaction] {
    fn hash(&self) -> hash::Hash {
        let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
        for txn in self {
            let serialized = bincode::serialize(txn).unwrap();
            ctx.update(&serialized);
        }
        let digest = ctx.finish();
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[0..32].clone_from_slice(digest.as_ref());
        return raw_hash.into();
    }
}

#[cfg(test)]
mod tests {
    use super::super::address;
    use super::super::hash;
    use super::super::hash::Hashable;
    use super::Transaction;

    #[test]
    fn hash() {
        let txns: [Transaction; 3] = [
            Transaction {
                source: address::Address([1; 20]),
                destination: address::Address([1; 20]),
                amount: 40,
            },
            Transaction {
                source: address::Address([2; 20]),
                destination: address::Address([3; 20]),
                amount: 90,
            },
            Transaction {
                source: address::Address([4; 20]),
                destination: address::Address([5; 20]),
                amount: 120,
            },
        ];
        let hash = txns.hash();
        let should_be = hash::Hash(hex!(
            "b22d7cf2e0d5996a88ab5334f0ecaa4b6a0464f1de227700f93bb6aefa4f8e01"
        ));
        assert_eq!(hash, should_be);
    }
}
