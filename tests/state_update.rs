use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::{self, miner::memory_pool, state, wallet};
use rand::Rng;
use std::sync::{mpsc, Arc, Mutex};
use prism::handler::{confirm_new_tx_block_transactions, unconfirm_old_tx_block_transactions};
use prism::state::UTXODatabase;
use prism::wallet::Wallet;
use prism::miner::memory_pool::MemoryPool;

// suppose the ledger confirms a new tx, suppose tx is from a sanitized tx block
fn ledger_new_txs(
    txs: Vec<Transaction>,
    mempool: &Mutex<MemoryPool>,
    state_db: &Mutex<UTXODatabase>,
    wallets: &Vec<Arc<Mutex<Wallet>>>,
) {
    let mut m = mempool.lock().unwrap();
    for tx in txs.iter() {
        for input in tx.input.iter() {
            m.remove_by_input(input);
        }
    }
    drop(m);
    confirm_new_tx_block_transactions(vec![txs], state_db, wallets);
//    let (to_delete, to_insert) = to_utxo(&tx);
//    if state_db.update(&to_delete, &to_insert).is_err() {
//        panic!("State DB error.");
//    }
//    for w in wallets {
//        let mut w = w.lock().unwrap();
//        w.update(&to_delete, &to_insert);
//    }
}

// suppose a miner mine the whole mempool, and they are confirmed in ledger
fn mine_whole_mempool(
    mempool: &Mutex<MemoryPool>,
    state_db: &Mutex<UTXODatabase>,
    wallets: &Vec<Arc<Mutex<Wallet>>>,
) {
    let m = mempool.lock().unwrap();
    let len = m.len();
    let txs = m.get_transactions(len);
    drop(m);
    ledger_new_txs(txs, mempool, state_db, wallets);
}

#[test]
fn wallets_pay_eachother() {
    //TODO: also need to test rollback
    const NUM: usize = 9;
    // initialize all sorts of stuff
    let state_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
    let state_db = UTXODatabase::new(state_path).unwrap();
    let state_db = Arc::new(Mutex::new(state_db));

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let (ctx_update_sink, _ctx_update_source) = mpsc::channel();

    let mut wallets = vec![];
    let mut addrs = vec![];
    //let's generate
    for _ in 0..NUM {
        let mut w = wallet::Wallet::new(&mempool, ctx_update_sink.clone());
        w.generate_keypair(); //add_keypair(our_addr);
        let addr: H256 = w.get_pubkey_hash().unwrap();
        wallets.push(Arc::new(Mutex::new(w)));
        addrs.push(addr);
    }

    // fund-raising, give every wallet 100
    let funding = Transaction {
        input: vec![],
        output: addrs
            .iter()
            .map(|addr| Output {
                value: 100,
                recipient: addr.clone(),
            })
            .collect(),
        signatures: vec![],
    };
    ledger_new_txs(
        vec![funding],
        &mempool,
        &state_db,
        &wallets,
    );
    println!(
        "Balance of wallets: {:?}.",
        wallets.iter().map(|w| w.lock().unwrap().balance()).collect::<Vec<u64>>()
    );
    println!("UTXO num: {}", state_db.lock().unwrap().num_utxo());
    for _ in 0..10 {
        //        // A --2--> B, B --1--> A. Balance should be: A=A-1, B=B+1
        let mut rng = rand::thread_rng();
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
        mine_whole_mempool(&mempool, &state_db, &wallets);
        println!(
            "Balance of wallets: {:?}.",
            wallets.iter().map(|w| w.lock().unwrap().balance()).collect::<Vec<u64>>()
        );
        println!("UTXO num: {}", state_db.lock().unwrap().num_utxo());
        for w in wallets.iter() {
            let mut balance_in_state = 0u64;
            let w = w.lock().unwrap();
            for coin_id in w.get_coin_id().iter() {
                let coin_data = state_db.lock().unwrap().get(coin_id).unwrap().unwrap();
                balance_in_state += coin_data.value;
            }
            assert_eq!(
                balance_in_state,
                w.balance(),
                "state and wallet not compatible"
            );
        }
    }

    drop(state_db.lock().unwrap());
    drop(state_db);
    assert!(rocksdb::DB::destroy(
        &rocksdb::Options::default(),
        "/tmp/prism_test_state.rocksdb"
    )
    .is_ok());
}
