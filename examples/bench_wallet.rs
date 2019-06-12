use prism::block::Block;
//use prism::blockchain::utils as bc_utils;
use prism::transaction::Transaction;
use prism::utxodb::UtxoDatabase;
use prism::crypto::hash::Hashable;
use prism::experiment::ico;
use prism::wallet::Wallet;
use std::sync::mpsc;
use std::time::Instant;
use std::thread;
use std::sync::Arc;
use log::{debug, error, info};
use std::process;

const REPEAT: usize = 2000;
const TX_COUNT: usize = 100;

fn main() {

  // init wallet database
  // init utxo database
  let utxodb = UtxoDatabase::new("/tmp/bench_utxodb.rocksdb").unwrap();
  let utxodb = Arc::new(utxodb);


  let wallet = Wallet::new("/tmp/bench_wallet2.rocksdb").unwrap();
  let wallet = Arc::new(wallet);

  // load wallet keys
  let key_path = "testbed/0.pkcs8";
  let content = match std::fs::read_to_string(&key_path) {
    Ok(c) => c,
    Err(e) => {
      println!("Error loading key pair at {}: {}", &key_path, &e);
      process::exit(1);
    }
  };
  let decoded = match base64::decode(&content.trim()) {
    Ok(d) => d,
    Err(e) => {
      println!("Error decoding key pair at {}: {}", &key_path, &e);
      process::exit(1);
    }
  };
  let keypair = prism::crypto::sign::KeyPair::from_pkcs8(decoded);
  match wallet.load_keypair(keypair) {
    Ok(a) => info!("Loaded key pair for address {}", &a),
    Err(e) => {
      println!("Error loading key pair into wallet: {}", &e);
      process::exit(1);
    }
  }

  //ico
  ico(&wallet.addresses().unwrap(), &utxodb, &wallet);
  println!("The wallet starts with {} coins.", wallet.number_of_coins());
  let mut rng = rand::thread_rng();

  let start = Instant::now();
  let mut update_time: f64 = 0.0;
  let addr = wallet.addresses().unwrap()[0];
  let value: u64 = 100; //rng.gen_range(10, 20);
  let mut prev_coin = None;

  for i in 1..(REPEAT+1) {
    let mut txs: Vec<Transaction> = vec![];

    for _ in 0..TX_COUNT {
      match wallet.create_transaction(addr, value, prev_coin) {
        Ok(tx) => {
          prev_coin = Some(tx.input.last().unwrap().clone());
          txs.push(tx);
        },
        Err(e) => {prev_coin = None},//println!(" Out of coins {}", e)
      }

    }
//    println!("The wallet {} coins after {} repeat", wallet.number_of_coins(), i);
    let update_start = Instant::now();
    let coin_diff = utxodb.apply_diff(&txs, &[]).unwrap();
    wallet.apply_diff(&coin_diff.0, &coin_diff.1).unwrap();
    let update_end = Instant::now();
    update_time += update_end.duration_since(update_start).as_micros() as f64;


    let end = Instant::now();
    let time = end.duration_since(start).as_micros() as f64 - update_time;
    println!("Rate_total {} \n", (((i+1)*TX_COUNT) as f64)*1000000.0/time);
//    println!("The wallet {} coins after {} repeat and update. Valid txs {} \n", wallet.number_of_coins(), i, coin_diff.0.len());

  }

  let end = Instant::now();
  let time = end.duration_since(start).as_micros() as f64 - update_time;
  println!("Total time {}", time/1000000.0 );

//  let start = Instant::now();
//  let addr = wallet.addresses().unwrap()[0];
//  let value: u64 = 100; //rng.gen_range(10, 20);
//  wallet.create_transactions(&[(addr, value); TX_COUNT*REPEAT]);
//  let end = Instant::now();
//  let time = end.duration_since(start).as_micros() as f64;
//  println!("Total time {}", time/1000000.0 );
}
