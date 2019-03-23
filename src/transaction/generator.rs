// Randomly generates objects of the given class

use crate::crypto;
use super::{Transaction, Input, Output, Signature};
use rand::{Rng, RngCore};

pub fn random() -> Transaction {
    let mut rng = rand::thread_rng();
    let num_inputs =  rng.gen_range(1, 5);
    let input :Vec<Input> = (0..num_inputs).map(|_| tx_input()).collect();
    let num_outputs =  rng.gen_range(1, 5);
    let output :Vec<Output> = (0..num_outputs).map(|_| tx_output()).collect();
    let signatures: Vec<Signature> = vec![]; // TODO: Add signatures
    return Transaction {
        input, output, signatures
    };
}

fn tx_input() -> Input {
    let mut rng = rand::thread_rng();
    let hash = crypto::generator::h256();
    let index = rng.next_u32();
    return Input {hash, index};
}


fn tx_output() -> Output {
    let mut rng = rand::thread_rng();
    let value = rng.next_u64();
    let recipient = crypto::generator::h256();
    return Output {value, recipient};
}
