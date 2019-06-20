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

const REPEAT: usize = 1000;
const TX_COUNT: usize = 20000;

fn main() {

  // init wallet database
  // init utxo database
  let utxodb = UtxoDatabase::new("/tmp/bench_utxodb.rocksdb").unwrap();
  let utxodb = Arc::new(utxodb);


  let wallet = Wallet::new("/tmp/bench_wallet2.rocksdb").unwrap();
  let wallet = Arc::new(wallet);

  // load wallet keys
  {
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
  }

  //ico
  ico(&wallet.addresses().unwrap(), &utxodb, &wallet, TX_COUNT+10, 100);
  println!("The wallet starts with {} coins.", wallet.number_of_coins());
  let mut rng = rand::thread_rng();

  let start = Instant::now();
  let mut tx_generate_time: f64 = 0.0;
  let mut utxo_update_time: f64 = 0.0;
  let mut wallet_update_time: f64 = 0.0;
  let addr = wallet.addresses().unwrap()[0];
  let value: u64 = 100; //rng.gen_range(10, 20);
  let mut prev_coin = None;

  for i in 1..=REPEAT {
    let mut txs: Vec<Transaction> = vec![];
    let mut j: usize = 0;

    //1. Generate txs
    let tx_generate_start = Instant::now();
    loop {
      match wallet.create_transaction(addr, value, prev_coin) {
        Ok(tx) => {
          prev_coin = Some(tx.input.last().unwrap().clone());
          txs.push(tx);
          j += 1;
        },
        Err(e) => {
          prev_coin = None;
//          print!(" Out of coins in iteration {}. Error: {}", i*TX_COUNT+j, e)
        }
      }
      if j >= TX_COUNT {
        break;
      }
    }
    let tx_generate_end = Instant::now();
    utxo_update_time += tx_generate_end.duration_since(tx_generate_start).as_micros() as f64;

    // 2. Pass the tx through utxo
    let utxo_update_start = Instant::now();
    let coin_diff = utxodb.apply_diff(&txs, &[], None).unwrap();
    let utxo_update_end = Instant::now();
    tx_generate_time += utxo_update_end.duration_since(utxo_update_start).as_micros() as f64;

    //3. Pass the coin diff through wallet
    let wallet_update_start = Instant::now();
    wallet.apply_diff(&coin_diff.0, &coin_diff.1).unwrap();
    let wallet_update_end = Instant::now();
    wallet_update_time += wallet_update_end.duration_since(wallet_update_start).as_micros() as f64;


    println!("{} Wallet tx gen rate {}",i, ((i*TX_COUNT) as f64)*1000_000.0/tx_generate_time);
    println!("{} Utxodb apply diff rate {}",i, ((i*TX_COUNT) as f64)*1000_000.0/utxo_update_time);
    println!("{} Wallet apply diff rate {} \n",i, ((i*TX_COUNT) as f64)*1000_000.0/wallet_update_time);
//    println!("The wallet {} coins after {} repeat .Tx len {}. Coin diff {}:{}\n", wallet.number_of_coins(), i, txs.len(), coin_diff.0.len(), coin_diff.1.len());
  }

  let end = Instant::now();
  let time = end.duration_since(start).as_micros() as f64 - utxo_update_time;
  println!("Total time {}", time/1000000.0 );

}
