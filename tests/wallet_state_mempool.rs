use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::{self, miner::memory_pool, wallet, state};
use std::sync::{Arc, Mutex, mpsc};
use rand::Rng;

// suppose the ledger confirms a new tx, suppose tx is from a sanitized tx block
fn ledger_new_tx(tx: Transaction, mempool: &Mutex<memory_pool::MemoryPool>, wallets: &mut Vec<wallet::Wallet>) {
    let mut m = mempool.lock().unwrap();
    for input in tx.input.iter() {
        m.remove_by_input(input);
    }
    drop(m);
    //statedb.receive(&tx);//we don't have this func now
    for w in wallets.iter_mut() {
        w.receive(&tx);
    }
}

// suppose a miner mine the whole mempool, and they are confirmed in ledger
fn mine_whole_mempool(mempool: &Mutex<memory_pool::MemoryPool>, wallets: &mut Vec<wallet::Wallet>) {
    let m = mempool.lock().unwrap();
    let len = m.len();
    let txs = m.get_transactions(len);
    drop(m);
    for tx in txs {
        ledger_new_tx(tx,mempool, wallets);
    }
}

#[test]
fn wallets_pay_eachother() {//also need to test rollback
    const NUM: usize = 3;
    // initialize all sorts of stuff
    let state_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
    let statedb = state::UTXODatabase::new(state_path).unwrap();
    let statedb = Arc::new(statedb);

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let (ctx_update_sink, _ctx_update_source) = mpsc::channel();

    let mut wallets = vec![];
    let mut addrs = vec![];
    //let's generate
    for _ in 0..NUM {
        let mut w = wallet::Wallet::new(&mempool, ctx_update_sink.clone());
        w.generate_keypair();//add_keypair(our_addr);
        let addr: H256 = w.get_pubkey_hash().unwrap();
        wallets.push(w);
        addrs.push(addr);
    }

    // fund-raising, give every wallet 100
    let funding = Transaction {
        input: vec![],
        output: addrs.iter().map(|addr|Output{value: 100, recipient: addr.clone()}).collect(),
        signatures: vec![],
    };
    ledger_new_tx(funding, &mempool, &mut wallets);
    println!("Balance of wallets: {:?}.", wallets.iter().map(|w|w.balance()).collect::<Vec<u64>>());
    for _ in 0..10 {
//        // A --2--> B, B --1--> A. Balance should be: A=A-1, B=B+1
        let mut rng = rand::thread_rng();
        let payer: usize = rng.gen_range(0, NUM);
        let mut receiver: usize = rng.gen_range(0, NUM);
        while payer == receiver {
            receiver = rng.gen_range(0, NUM);
        }
        let v: u64 = rng.gen_range(1, 5);
        assert!(wallets[payer].pay(addrs[receiver].clone(), v).is_ok());
        println!("Payment: {} to {}, value {}.", payer, receiver, v);
        println!("Dummy mining, sanitization and ledger generation");
        mine_whole_mempool(&mempool,  &mut wallets);
        println!("Balance of wallets: {:?}.", wallets.iter().map(|w|w.balance()).collect::<Vec<u64>>());
    }

    drop(statedb);
    assert!(rocksdb::DB::destroy(&rocksdb::Options::default(), "/tmp/prism_test_state.rocksdb").is_ok());
}
