use bincode::{serialize, deserialize};
use crate::crypto::hash::{Hashable, H256};
//use std::time::{SystemTime, UNIX_EPOCH};
use std;
use std::fmt;

/*
ToDo: Encoder and decoder for the transaction?
*/
pub type PubKey = u8;//will be changed
pub type Nonce = u32;
pub type Signature = u8;
pub type FromPair = (PubKey, Nonce);
pub type ToPair = (PubKey, u32);

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Action {
    from: Vec<FromPair>,
    to: Vec<ToPair>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Transaction {
    timestamp: u64,
    fee: u32,
    action: Action,
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        // ToDo: Serialize the object into a byte array
        // return the SHA256 of the byte array
        let x: [u8; 32] = [0; 32];
        return H256(x);
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UnverifiedTransaction {
    hash: H256,
    unsigned: Transaction,
    //some crypto fields?
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SignedTransaction {
    transaction: UnverifiedTransaction,
    signature: Signature,//what should be the sign type?
}

impl SignedTransaction {
//    pub fn new(timestamp: u64, nonce: u32, fee: u32, from: & Vec, to: & Vec) -> Self {
//        //TODO: New tx
//        let tx = Transaction {timestamp:1, nonce:1, fee:1, from:vec![(1,2)], to:vec![(3,4)]};//fake tx
//        let unverified = UnverifiedTransaction { unsigned: tx, hash: None};
//        let sign = 0u8;//fake signature
//        SignedTransaction {transaction: unverified, signature: sign}
//    }
//
    pub fn create(action: Action, fee: u32) -> Self {
        let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                panic!("SystemTime before UNIX EPOCH!");
                0u64
            },
        };
        let tx = Transaction { timestamp, fee, action };
        let hash = tx.hash();
        let unverified = UnverifiedTransaction { unsigned: tx, hash};
        let signature = 0 as Signature;//fake signature
        SignedTransaction {transaction: unverified, signature}
    }

}

impl fmt::Display for SignedTransaction  {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SignedTransaction {{ ")?;
        write!(f, "{:?}", self.transaction.unsigned);
        write!(f, " }}")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let action = Action {from:vec![(1, 2)], to:vec![(3, 4)]};
        let tx: SignedTransaction = SignedTransaction::create(action, 1);
        println!("{}",tx);
        let encoded: Vec<u8> = serialize(&tx).unwrap();
        //println!("{:?}",encoded);
        let decoded: SignedTransaction = deserialize(&encoded[..]).unwrap();
        println!("{}",decoded);
    }
}
