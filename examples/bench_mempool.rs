//use prism::block::Block;
////use prism::blockchain::utils as bc_utils;
//use prism::transaction::Transaction;
//use prism::miner::memory_pool::MemoryPool;
//use prism::network::server;
//use prism::crypto::hash::{Hashable,H256};
//
//use std::sync::mpsc;
//use std::time::Instant;
//use std::thread;
//use std::net;
//use std::process;
//use std::sync::Arc;
//use std::sync::Mutex;
//
//const TX_COUNT: u32 = 100000;
//const TX_SIZE: f64 = 250.0;
//
//#[cfg(feature = "test-utilities")]
fn main() {
    //    use prism::transaction::tests::generate_random_transaction;
    //
    //
    //    // parse p2p server address
    //    let p2p_addr = "127.0.0.1:8000".parse::<net::SocketAddr>().unwrap_or_else(|e| {
    //        panic!("Error parsing P2P server address: {}", e);
    //        process::exit(1);
    //    });
    //
    //    // initialize mempool
    //    let mempool = MemoryPool::new();
    //    let mempool = Arc::new(std::sync::Mutex::new(mempool));
    //
    //
    //    let mut txs: Vec<(Transaction, H256)> = vec![];
    //    let start = Instant::now();
    //    for i in 0..TX_COUNT{
    //        let tx = generate_random_transaction();
    //        let hash = H256::default();
    //        txs.push((tx, hash));
    //    }
    //    let end = Instant::now();
    //    let time = end.duration_since(start).as_micros() as f64;
    //    println!("Tx generation rate {}", (TX_COUNT as f64)*TX_SIZE/time);
    //
    //    let start = Instant::now();
    ////    for tx in txs.iter(){
    ////        new_transaction(tx.clone(), &mempool)
    ////    }
    ////
    //    new_transaction_vec(txs, &mempool);
    //    let end = Instant::now();
    //    let time = end.duration_since(start).as_micros() as f64;
    //    println!("Mempool insertion rate {}", (TX_COUNT as f64)*TX_SIZE/time);
    //
}
//
//
//fn new_transaction(transaction: Transaction, mempool: &Mutex<MemoryPool>) {
//    let mut mempool = mempool.lock().unwrap();
//    // memory pool check
//    if !mempool.contains(&transaction.hash()) && !mempool.is_double_spend(&transaction.input) {
//        // if check passes, insert the new transaction into the mempool
////        server.broadcast(Message::NewTransactionHashes(vec![transaction.hash()]));
//        mempool.insert(transaction);
//    }
//    drop(mempool);
//}
//
//fn new_transaction_vec(transaction: Vec<(Transaction, H256)>, mempool: &Mutex<MemoryPool>) {
//    let mut mempool = mempool.lock().unwrap();
//    // memory pool check
//    for (tx, hash) in transaction.iter() {
//        bench_utxodb.rs        if !mempool.contains(&tx.hash()) && !mempool.is_double_spend(&tx.input) {
//            // if check passes, insert the new transaction into the mempool
////        server.broadcast(Message::NewTransactionHashes(vec![transaction.hash()]));
////            mempool.insert_with_hash(tx.clone(), *hash);
//            mempool.insert(tx.clone());
//        }
//    }
//    drop(mempool);
//}
