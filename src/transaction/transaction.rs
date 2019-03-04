
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, deserialize};
use crate::crypto::hash::{Hashable, H256};



/*
ToDo: Encoder and decoder for the transaction?
*/
type PubKey = u8;
type Nonce = u32;

type FromPair = (PubKey, Nonce);//the pubkey, and box_nonce of the from boxes
type ToPair = (PubKey, u32);//the pubkey, and values

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


#[derive(Debug)]
pub struct UnverifiedTransaction {
    unsigned: Transaction,
    hash: H256,//or modifier id?
    //some crypto fields?
}

#[derive(Debug)]
pub struct SignedTransaction {
    transaction: UnverifiedTransaction,
    signature: u8,//what should be the sign type?
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
//    pub fn create(to: & Vec) -> Self {
//        //TODO: Should find all unspent coins and use them to create a tx
//        let tx = Transaction {timestamp:1, nonce:1, fee:1, from:vec![(1,2)], to:vec![(3,4)]};//fake tx
//        let unverified = UnverifiedTransaction { unsigned: tx, hash: None};
//        let sign = 0u8;//fake signature
//        SignedTransaction {transaction: unverified, signature: sign}
//    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let action = Action {from:vec![(1, 2)], to:vec![(3, 4)]};
        let tx = Transaction {timestamp:1, fee:1, action };
        println!("{:?}",tx);
        let encoded: Vec<u8> = serialize(&tx).unwrap();
        //println!("{:?}",encoded);
        let decoded: Transaction = deserialize(&encoded[..]).unwrap();
        println!("{:?}",decoded);
    }
}