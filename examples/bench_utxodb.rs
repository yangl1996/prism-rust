//use prism::block::Block;
////use prism::blockchain::utils as bc_utils;
//use prism::transaction::Transaction;
//use prism::utxodb::UtxoDatabase;
//use prism::crypto::hash::Hashable;
//use std::sync::mpsc;
//use std::time::Instant;
//use std::thread;
//use std::sync::Arc;
//
//const TX_COUNT: u32 = 10000;
//const TX_SIZE: f64 = 250.0;
//
//#[cfg(feature = "test-utilities")]
fn main() {
    //  use prism::transaction::tests::generate_random_transaction;
    //
    //  let utxodb = UtxoDatabase::new("/tmp/prism/bench_utxodb.rocksdb").unwrap();
    //  let utxo_arc = Arc::new(utxodb);
    //
    //  let mut tx: Vec<Transaction> = vec![];
    //  let start = Instant::now();
    //  for i in 0..TX_COUNT*100{
    //    tx.push(generate_random_transaction());
    //  }
    //  let end = Instant::now();
    //  let time = end.duration_since(start).as_micros() as f64;
    //  println!("Tx generation rate {}", (TX_COUNT as f64)*TX_SIZE/time);
    //
    //  let start = Instant::now();
    //  let utxo_clone= Arc::clone(&utxo_arc);
    //
    //  help(&utxo_clone, &tx);
    //  let end = Instant::now();
    //  let time = end.duration_since(start).as_micros() as f64;
    //  println!("Utxo update rate {}", (TX_COUNT as f64)*TX_SIZE/time);
    //
    //
    //  let mut txs: Vec<Vec<Transaction>> = vec![];
    //  for j in 0..10 {
    //    let mut tx: Vec<Transaction> = vec![];
    //    for i in 0..TX_COUNT {
    //      tx.push(generate_random_transaction());
    //    }
    //    txs.push(tx);
    //  }
    //  let start = Instant::now();
    //
    //  for j in 0..10 {
    //    let utxo_clone= Arc::clone(&utxo_arc);
    ////        let tx = Arc::new(txs[j]);
    //    let tx = txs[j].clone();
    ////        help(&utxo_clone, &tx);
    //    thread::spawn(move || {help(&utxo_clone, &tx)} );
    //  }
    //  let end = Instant::now();
    //  let time = end.duration_since(start).as_micros() as f64;
    //  println!("Utxo update rate {}", (TX_COUNT as f64)*TX_SIZE*10.0/time);
    //
}
//
//fn help(utxodb: &UtxoDatabase, tx: &Vec<Transaction>) {
//  utxodb.apply_diff(tx, &vec![]);
//}
