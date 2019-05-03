use prism::crypto::hash::H256;
use prism::handler::{confirm_new_tx_block_transactions, unconfirm_old_tx_block_transactions};
use prism::miner::memory_pool::MemoryPool;
use prism::utxodb::UtxoDatabase;
use prism::transaction::{Output, Transaction};
use prism::wallet::Wallet;
use rand::Rng;
use std::sync::{mpsc, Arc, Mutex};
use prism::crypto::hash::tests::generate_random_hash;
// suppose the ledger confirms a new tx, suppose tx is from a sanitized tx block
fn ledger_new_txs(
    txs: Vec<Transaction>,
    mempool: &Mutex<MemoryPool>,
    utxodb: &UtxoDatabase,
    wallets: &Vec<Mutex<Wallet>>,
) {
    let mut m = mempool.lock().unwrap();
    for tx in txs.iter() {
        for input in tx.input.iter() {
            m.remove_by_input(input);
        }
    }
    drop(m);
    confirm_new_tx_block_transactions(vec![txs], utxodb, wallets);
}

// suppose a miner mine the whole mempool, and they are confirmed in ledger
fn mine_whole_mempool(
    mempool: &Mutex<MemoryPool>,
    utxodb: &UtxoDatabase,
    wallets: &Vec<Mutex<Wallet>>,
) {
    let m = mempool.lock().unwrap();
    let len = m.len();
    let txs = m.get_transactions(len);
    drop(m);
    ledger_new_txs(txs, mempool, utxodb, wallets);
}

fn status_check(utxodb: &UtxoDatabase, wallets: &Vec<Mutex<Wallet>>) {
    println!(
        "Balance of wallets: {:?}.",
        wallets
            .iter()
            .map(|w| w.lock().unwrap().balance().unwrap())
            .collect::<Vec<u64>>()
    );
//    println!("UTXO num: {}", utxodb.lock().unwrap().num_utxo());
    for w in wallets.iter() {
        let mut balance_in_state = 0u64;
        let w = w.lock().unwrap();
        for coin_id in w.get_coin_id().iter() {
            let coin_data = utxodb.get(coin_id).unwrap().unwrap();
            balance_in_state += coin_data.value;
        }
        assert_eq!(
            balance_in_state,
            w.balance().unwrap(),
            "state and wallet not compatible"
        );
    }
}

#[test]
fn wallet_keep_paying() {
    //TODO: also need to test rollback
    const NUM: usize = 1;
    const ITER: usize = 50;
    // initialize all sorts of stuff
    let utxodb_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
    let utxodb = UtxoDatabase::new(utxodb_path).unwrap();
    let utxodb = Arc::new(Mutex::new(utxodb));

    let mempool = MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let (ctx_update_sink, _ctx_update_source) = mpsc::channel();

    let mut wallets = vec![];
    let mut addrs = vec![];
    for _ in 0..1 {
        let mut w = Wallet::new(std::path::Path::new("/tmp/walletdb.rocksdb"),&mempool, ctx_update_sink.clone()).unwrap();
        w.generate_keypair().unwrap(); //add_keypair(our_addr);
        let addr: H256 = w.get_an_address().unwrap();
        wallets.push(Mutex::new(w));
        addrs.push(addr);
    }
    let wallets = Arc::new(wallets);
    // fund-raising, give every wallet 100*100
    let funding = Transaction {
        input: vec![],
        output: addrs
            .iter()
            .map(|addr| {
                (0..100).map(move |_| Output {
                    value: 100,
                    recipient: addr.clone(),
                })
            })
            .flatten()
            .collect(),
        authorization: vec![],
    };
    ledger_new_txs(vec![funding], &mempool, &utxodb, &wallets);
    status_check(&utxodb, &wallets);
    let mut rng = rand::thread_rng();
    // test payment for some iterations
    for _ in 0..ITER {
        let payer: usize = rng.gen_range(0, NUM);
        let v: u64 = rng.gen_range(1, 5);
        let mut w = wallets[payer].lock().unwrap();
        assert!(w.pay(generate_random_hash(), v).is_ok());
        drop(w);
        println!("Payment: {} to trash, value {}.", payer, v);
        println!("Dummy mining, sanitization and ledger generation");
        mine_whole_mempool(&mempool, &utxodb, &wallets);
        status_check(&utxodb, &wallets);
    }
    //this iteration is for test of rollback
    for _ in 0..ITER {
        let payer: usize = rng.gen_range(0, NUM);
        let v: u64 = rng.gen_range(1, 5);
        let mut w = wallets[payer].lock().unwrap();
        assert!(w.pay(generate_random_hash(), v).is_ok());
        drop(w);
        println!("Payment: {} to trash, value {}.", payer, v);
    }
    println!("Dummy mining, sanitization and ledger generation");
    let m = mempool.lock().unwrap();
    let len = m.len();
    let txs = m.get_transactions(len);
    drop(m);
    ledger_new_txs(txs.clone(), &mempool, &utxodb, &wallets);
    status_check(&utxodb, &wallets);
    // rollback txs
    println!("Rollback past transactions");
    unconfirm_old_tx_block_transactions(vec![txs], &utxodb, &wallets);
    status_check(&utxodb, &wallets);

    drop(utxodb);
    assert!(rocksdb::DB::destroy(
        &rocksdb::Options::default(),
        "/tmp/prism_test_state.rocksdb"
    )
    .is_ok());
    drop(wallets[0].lock().unwrap());
    drop(wallets);
    println!("{:?}",rocksdb::DB::destroy(
        &rocksdb::Options::default(),
        "/tmp/walletdb.rocksdb"
    ));
}
