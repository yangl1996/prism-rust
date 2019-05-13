#[macro_use]
extern crate clap;

use prism::blockdb::BlockDatabase;
use prism::crypto::hash::H256;

use std::io;
use std::io::prelude::*;

fn main() {
    let matches = clap_app!(Prism =>
     (@arg block_db: --blockdb [PATH] "Sets the path of the block database")
    )
        .get_matches();
    const DEFAULT_BLOCKDB: &str = "/tmp/prism-blocks.rocksdb";
    let db = match matches.value_of("block_db") {
        Some(path) => BlockDatabase::restore(&path).unwrap(),
        None => BlockDatabase::restore(&DEFAULT_BLOCKDB).unwrap(),
    };

    let stdin = io::stdin();
    print!("Block hash: ");io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        let input = line.unwrap();
        let vec_bytes = hex::decode(input).expect("Decoding failed");
        let mut raw_bytes = [0; 32];
        raw_bytes.copy_from_slice(&vec_bytes);
        let hash: H256 = (&raw_bytes).into();
        match db.get(&hash) {
            Ok(Some(block)) => println!("{:?}", block),
            _ => println!("No such block"),
        }
        print!("\nBlock hash: ");io::stdout().flush().unwrap();
    }
}