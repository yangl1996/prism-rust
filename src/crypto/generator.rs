/*
Generates random instances of the given class
*/

use super::hash::{Hashable, H256};
use super::merkle::MerkleTree;
use rand::{Rng, RngCore};

/// Generates a merkle tree of random H256.
pub fn merkle_tree<'a>(size: u32) -> MerkleTree<'a, H256> {
    unimplemented!();
}

/// Generates a random H256 hash.
pub fn h256() -> H256 {
    let u8_array: [u8; 32] = u8_32_array();
    let hash : H256 = (&u8_array).into();
    return hash;
}

/// Generates a random 32 element array of u8 type.
fn u8_32_array() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let ran: Vec<u8> = (0..32).map(|_| rng.gen_range(0, 255) as u8).collect();
    return from_slice(&ran);
}

fn from_slice(bytes: &[u8]) -> [u8; 32] {
    let mut array = [0; 32];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    return array;
}
