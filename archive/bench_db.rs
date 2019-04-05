use prism::blockdb::{Result, BlockDatabase};
use prism::block::generator as bgen;
use prism::crypto::generator as cgen;
use prism::crypto::hash::Hashable;
use std::time::Instant;

const REPEAT: usize = 100;

fn main() {
    // prepare block data
    let proposer = bgen::prop_block2();
    let voter = bgen::voter_block(2);
    let transaction = bgen::tx_block();

    // prepare random hashes, so that we can insert those blocks multiple times
    let mut proposer_hash = vec![];
    let mut voter_hash = vec![];
    let mut transaction_hash = vec![];

    println!("Proposer size: {} bytes", bincode::serialize(&proposer).unwrap().len());
    println!("Voter size: {} bytes", bincode::serialize(&voter).unwrap().len());
    println!("Transaction size: {} bytes", bincode::serialize(&transaction).unwrap().len());

    for _ in 0..REPEAT {
        proposer_hash.push(cgen::h256());
        voter_hash.push(cgen::h256());
        transaction_hash.push(cgen::h256());
    }

    let db = BlockDatabase::new(std::path::Path::new("/tmp/prism_bench")).unwrap();

    // insert data
    println!("Inserting data");
    let start = Instant::now();
    for i in 0..REPEAT {
        db.insert(&proposer_hash[i], &proposer);
    }
    let end = Instant::now();

    let time = end.duration_since(start).as_micros() as f64;
    let throughput = REPEAT as f64 / (time / 1000000.0);
    println!("Insert - proposer: {:.2} blks/s", throughput);

    let start = Instant::now();
    for i in 0..REPEAT {
        db.insert(&voter_hash[i], &voter);
    }
    let end = Instant::now();

    let time = end.duration_since(start).as_micros() as f64;
    let throughput = REPEAT as f64 / (time / 1000000.0);
    println!("Insert - voter: {:.2} blks/s", throughput);

    let start = Instant::now();
    for i in 0..REPEAT {
        db.insert(&transaction_hash[i], &transaction);
    }
    let end = Instant::now();

    let time = end.duration_since(start).as_micros() as f64;
    let throughput = REPEAT as f64 / (time / 1000000.0);
    println!("Insert - transaction: {:.2} blks/s", throughput);

    drop(db);
    rocksdb::DB::destroy(&rocksdb::Options::default(), "/tmp/prism_bench");
}
