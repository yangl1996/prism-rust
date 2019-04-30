// Randomly generates objects of the given class

use super::crypto as crypto_generator;
use crate::crypto;
use crate::transaction::{Authorization, CoinId, Input, Output, Transaction};
use rand::{Rng, RngCore};

pub fn random() -> Transaction {
    let mut rng = rand::thread_rng();
    let num_inputs = rng.gen_range(2, 3);
    let input: Vec<Input> = (0..num_inputs).map(|_| tx_input()).collect();
    let num_outputs = rng.gen_range(2, 3);
    let output: Vec<Output> = (0..num_outputs).map(|_| tx_output()).collect();
    let authorization: Vec<Authorization> = vec![]; // It's hard to generate valid signatures, so just don't put signatures here
    return Transaction {
        input,
        output,
        authorization,
    };
}

fn tx_input() -> Input {
    let mut rng = rand::thread_rng();
    let hash = crypto_generator::h256();
    let index = rng.next_u32();
    let value = rng.next_u64();
    let owner = crypto_generator::h256();
    let coin = CoinId { hash, index };
    return Input { coin, value, owner };
}

fn tx_output() -> Output {
    let mut rng = rand::thread_rng();
    let value = rng.gen_range(10, 30);
    let recipient = crypto_generator::h256();
    return Output { value, recipient };
}
