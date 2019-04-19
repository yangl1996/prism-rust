use prism::crypto::hash::H256;
use prism::handler::{confirm_new_tx_block_transactions, unconfirm_old_tx_block_transactions};
use prism::miner::memory_pool::MemoryPool;
use prism::state::UTXODatabase;
use prism::transaction::{Output, Transaction};
use prism::wallet::Wallet;
use rand::Rng;
use std::sync::{mpsc, Arc, Mutex};

// suppose the ledger confirms a new tx, suppose tx is from a sanitized tx block
fn ledger_new_txs(
    txs: Vec<Transaction>,
    mempool: &Mutex<MemoryPool>,
    utxodb: &Mutex<UTXODatabase>,
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
    utxodb: &Mutex<UTXODatabase>,
    wallets: &Vec<Mutex<Wallet>>,
) {
    let m = mempool.lock().unwrap();
    let len = m.len();
    let txs = m.get_transactions(len);
    drop(m);
    ledger_new_txs(txs, mempool, utxodb, wallets);
}

fn status_check(utxodb: &Mutex<UTXODatabase>, wallets: &Vec<Mutex<Wallet>>) {
    println!(
        "Balance of wallets: {:?}.",
        wallets
            .iter()
            .map(|w| w.lock().unwrap().balance())
            .collect::<Vec<u64>>()
    );
    println!("UTXO num: {}", utxodb.lock().unwrap().num_utxo());
    for w in wallets.iter() {
        let mut balance_in_state = 0u64;
        let w = w.lock().unwrap();
        for coin_id in w.get_coin_id().iter() {
            let coin_data = utxodb.lock().unwrap().get(coin_id).unwrap().unwrap();
            balance_in_state += coin_data.value;
        }
        assert_eq!(
            balance_in_state,
            w.balance(),
            "state and wallet not compatible"
        );
    }
}

#[test]
fn wallets_pay_eachother() {
    //TODO: also need to test rollback
    const NUM: usize = 3;
    const ITER: usize = 10;
    // initialize all sorts of stuff
    let utxodb_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
    let utxodb = UTXODatabase::new(utxodb_path).unwrap();
    let utxodb = Arc::new(Mutex::new(utxodb));

    let mempool = MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let (ctx_update_sink, _ctx_update_source) = mpsc::channel();

    let mut wallets = vec![];
    let mut addrs = vec![];
    for _ in 0..NUM {
        let mut w = Wallet::new(&mempool, ctx_update_sink.clone());
        w.generate_keypair(); //add_keypair(our_addr);
        let addr: H256 = w.get_pubkey_hash().unwrap();
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
        key_sig: vec![],
    };
    ledger_new_txs(vec![funding], &mempool, &utxodb, &wallets);
    status_check(&utxodb, &wallets);
    let mut rng = rand::thread_rng();
    // test payment for some iterations
    for _ in 0..ITER {
        let payer: usize = rng.gen_range(0, NUM);
        let mut receiver: usize = rng.gen_range(0, NUM);
        while payer == receiver {
            receiver = rng.gen_range(0, NUM);
        }
        let v: u64 = rng.gen_range(1, 5);
        let mut w = wallets[payer].lock().unwrap();
        assert!(w.pay(addrs[receiver].clone(), v).is_ok());
        drop(w);
        println!("Payment: {} to {}, value {}.", payer, receiver, v);
        println!("Dummy mining, sanitization and ledger generation");
        mine_whole_mempool(&mempool, &utxodb, &wallets);
        status_check(&utxodb, &wallets);
    }
    //this iteration is for test of rollback
    for _ in 0..ITER {
        let payer: usize = rng.gen_range(0, NUM);
        let mut receiver: usize = rng.gen_range(0, NUM);
        while payer == receiver {
            receiver = rng.gen_range(0, NUM);
        }
        let v: u64 = rng.gen_range(1, 5);
        let mut w = wallets[payer].lock().unwrap();
        assert!(w.pay(addrs[receiver].clone(), v).is_ok());
        drop(w);
        println!("Payment: {} to {}, value {}.", payer, receiver, v);
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

    drop(utxodb.lock().unwrap());
    drop(utxodb);
    assert!(rocksdb::DB::destroy(
        &rocksdb::Options::default(),
        "/tmp/prism_test_state.rocksdb"
    )
    .is_ok());
}
