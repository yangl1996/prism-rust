use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::{self, miner::memory_pool, wallet, state};
use std::sync::{Arc, Mutex, mpsc};

// suppose the ledger confirms a new tx
fn ledger_new_tx(tx: Transaction, mempool: &Mutex<memory_pool::MemoryPool>, wallet_1: &mut wallet::Wallet, wallet_2: &mut wallet::Wallet) {
    let mut m = mempool.lock().unwrap();
    for input in tx.input.iter() {
        m.remove_by_input(input);
    }
    drop(m);
    //statedb.receive(&tx);//we don't have this func now
    wallet_1.receive(&tx);
    wallet_2.receive(&tx);
}

// suppose a miner mine the whole mempool, and they are confirmed in ledger
fn mine_whole_mempool(mempool: &Mutex<memory_pool::MemoryPool>, wallet_1: &mut wallet::Wallet, wallet_2: &mut wallet::Wallet) {
    let m = mempool.lock().unwrap();
    let len = m.len();
    let txs = m.get_transactions(len);
    drop(m);
    for tx in txs {
        ledger_new_tx(tx,mempool, wallet_1, wallet_2);
    }
}

#[test]
fn wallet_state_mempool() {//also need to test rollback
    // initialize all sorts of stuff
    let state_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
    let statedb = state::UTXODatabase::new(state_path).unwrap();
    let statedb = Arc::new(statedb);

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let (ctx_update_sink, _ctx_update_source) = mpsc::channel();

    let mut wallet_1 = wallet::Wallet::new(&mempool, ctx_update_sink.clone());
    // get an address from the wallet
    wallet_1.generate_keypair();//add_keypair(our_addr);
    let addr_1: H256 = wallet_1.get_pubkey_hash().unwrap();

    let mut wallet_2 = wallet::Wallet::new(&mempool, ctx_update_sink.clone());
    // get an address from the wallet
    wallet_2.generate_keypair();//add_keypair(our_addr);
    let addr_2: H256 = wallet_2.get_pubkey_hash().unwrap();

    // fund-raising
    let funding = Transaction {
        input: vec![],
        output: vec![
            Output {value: 100, recipient: addr_1},
            Output {value: 100, recipient: addr_2},
        ],
        signatures: vec![],
    };
    ledger_new_tx(funding, mempool.as_ref(), &mut wallet_1, &mut wallet_2);
    assert_eq!(wallet_1.balance(), 100);
    assert_eq!(wallet_2.balance(), 100);

    for i in 1u64..100 {
        // A --2--> B, B --1--> A. Balance should be: A=A-1, B=B+1
        assert!(wallet_1.pay(addr_2, 2).is_ok());
        assert!(wallet_2.pay(addr_1, 1).is_ok());
        mine_whole_mempool(mempool.as_ref(), &mut wallet_1, &mut wallet_2);
        assert_eq!(wallet_1.balance(), 100-i);
        assert_eq!(wallet_2.balance(), 100+i);
    }
    //now balance A=1, B=199. We now B --199--> A

    assert!(wallet_2.pay(addr_1, 199).is_ok());
    mine_whole_mempool(mempool.as_ref(), &mut wallet_1, &mut wallet_2);
    assert_eq!(wallet_1.balance(), 200);
    assert_eq!(wallet_2.balance(), 0);

    drop(statedb);
    assert!(rocksdb::DB::destroy(&rocksdb::Options::default(), "/tmp/prism_test_state.rocksdb").is_ok());
}
