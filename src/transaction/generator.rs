/*
It randomly generates objects of the given class
*/
use crate::crypto::generator as crypto_generator;
use super::{Transaction,  Input, Output, Signature};
use rand::{Rng, RngCore};
type rgen = rand::prelude::ThreadRng;


pub fn transaction()  -> Transaction {
    let mut rng = rand::thread_rng();
    let input_size =  rng.gen_range(1, 5);
    let input :Vec<Input> = (0..input_size).map(|_| tx_input()).collect();
    let output_size =  rng.gen_range(1, 5);
    let output :Vec<Output> = (0..output_size).map(|_| tx_output()).collect();
    let signatures: Vec<Signature> = vec![]; // todo: Add signatures
    return Transaction {input, output, signatures};
}

pub fn tx_input() -> Input {
    let mut rng = rand::thread_rng();
    let hash = crypto_generator::H256();
    let index = rng.next_u32();
    return Input {hash, index};
}


pub fn tx_output() -> Output {
    let mut rng = rand::thread_rng();
    let value = rng.next_u64();
    let recipient = crypto_generator::H256();
    return Output {value, recipient};
}