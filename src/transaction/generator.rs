/*
It randomly generates objects of the given class
*/
use crate::crypto::generator as crypto_generator;
use super::{Transaction,  Input, Output, Signature};
use rand::{Rng, RngCore};
use crate::transaction::transaction_builder::TransactionBuilder;


pub fn transaction()  -> Transaction {
    let mut rng = rand::thread_rng();
    let input_number =  rng.gen_range(1, 5);
    let input :Vec<Input> = (0..input_number).map(|_| tx_input()).collect();
    let output_number =  rng.gen_range(1, 5);
    let output :Vec<Output> = (0..output_number).map(|_| tx_output()).collect();
    let signatures: Vec<Signature> = vec![]; // todo: Add signatures
    return Transaction {input, output, signatures};
}

pub fn tx_input() -> Input {
    let mut rng = rand::thread_rng();
    let hash = crypto_generator::h256();
    let index = rng.next_u32();
    return Input {hash, index};
}


pub fn tx_output() -> Output {
    let mut rng = rand::thread_rng();
    let value = rng.next_u64();
    let recipient = crypto_generator::h256();
    return Output {value, recipient};
}

pub fn random_transaction_builder() -> TransactionBuilder {
    let mut tb = TransactionBuilder::default();
    let mut rng = rand::thread_rng();
    for i in 0..rng.gen_range(1,5) {
        tb = tb.add_input(crypto_generator::h256(), rng.gen_range(1, 5));
    }
    for i in 0..rng.gen_range(1,5) {
        tb = tb.add_output(rng.gen_range(100,200), crypto_generator::h256());
    }
    tb
}