use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::visualization;
use prism::{self, blockchain, blockdb, miner::memory_pool, state, handler};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use rand::Rng;
use prism::wallet::WalletError;

const NUM_VOTER_CHAINS: u16 = 3;

fn main() {
    // initialize all sorts of stuff
    let blockdb_path = std::path::Path::new("/tmp/prism_itest_self_mining.rocksdb");
    let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = Arc::new(blockdb);

    let utxodb_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
    let utxodb = state::UTXODatabase::new(utxodb_path).unwrap();
    let utxodb = Arc::new(Mutex::new(utxodb));

    let (state_update_sink, state_update_source) = mpsc::channel();
    let blockchain = blockchain::BlockChain::new(NUM_VOTER_CHAINS, state_update_sink);
    let blockchain = Arc::new(Mutex::new(blockchain));

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let peer_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let peer_port = 12345;
    let peer_addr = std::net::SocketAddr::new(peer_ip, peer_port);

    let (_server, miner, wallets) =
        prism::start(peer_addr, &blockdb, &blockchain, &utxodb, &mempool, state_update_source).unwrap();

    let vis_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let vis_port = 8888;
    let vis_addr = std::net::SocketAddr::new(vis_ip, vis_port);
    visualization::Server::start(vis_addr, Arc::clone(&blockchain), Arc::clone(&blockdb), Arc::clone(&utxodb));

    // get the addr of the wallets
    let addrs: Vec<H256> = wallets.iter().map(|w| w.lock().unwrap().get_pubkey_hash().unwrap()).collect();

    // fund-raising, a 100 coin to the wallet
    let funding = Transaction {
        input: vec![],
        output: addrs
            .iter()
            .map(|addr| {
                (0..10).map(move |_| Output {
                    value: 100,
                    recipient: addr.clone(),
                })
            })
            .flatten()
            .collect(),
        key_sig: vec![],
    };
    handler::new_transaction(funding, &mempool);

    //this thread controls the miner to mine every 20ms.
    let _miner_clock_thread = std::thread::Builder::new().name("miner clock".to_string()).spawn(move || {
        for _ in 0..500 {
            miner.step();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        miner.exit();
    }).unwrap();
    //wait for some time, wait for initial tx get into ledger so our wallet can have money
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let mut balance_test = vec![1000u64; wallets.len()];
    // here we simulate a user who transfers some money to someone every .5s.
    let mut rng = rand::thread_rng();
    for i in 0..15 {
        println!(
            "At {} round, Balance of wallets: {:?}.",i,
            wallets
                .iter()
                .map(|w| w.lock().unwrap().balance())
                .collect::<Vec<u64>>()
        );
        let v: u64 = rng.gen_range(1, 10);
        let payer = rng.gen_range(0,wallets.len());
        match wallets[payer].lock().unwrap().pay((&[0u8;32]).into(), v) {
            Ok(hash) => {
                println!("The wallet {} pay to trash {} coin. payment successfully added to mempool, tx hash: {}", payer, v, hash);
                balance_test[payer] -= v;
            },
            Err(WalletError::ContextUpdateChannelError) => {
                panic!("cannot send miner context update, perhaps the miner is down?");
            },
            Err(_) => println!("payment error, perhaps last tx hasn't got into ledger"),
        };
        let receiver = rng.gen_range(0, wallets.len());
        if payer != receiver {
            let v: u64 = rng.gen_range(1, 10);
            match wallets[payer].lock().unwrap().pay(addrs[receiver], v) {
                Ok(hash) => {
                    println!("The wallet {} pay {}, {} coin. payment successfully added to mempool, tx hash: {}", payer, receiver, v, hash);
                    balance_test[payer] -= v;
                    balance_test[receiver] += v;
                },
                Err(WalletError::ContextUpdateChannelError) => {
                    panic!("cannot send miner context update, perhaps the miner is down?");
                },
                Err(_) => println!("payment error, perhaps last tx hasn't got into ledger"),
            };
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!(
        "At the end, Balance of wallets: {:?}.",
        wallets
            .iter()
            .map(|w| w.lock().unwrap().balance())
            .collect::<Vec<u64>>()
    );
    assert_eq!(wallets
                   .iter()
                   .map(|w| w.lock().unwrap().balance())
                   .collect::<Vec<u64>>(),
               balance_test,"balances are not as expected");
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
    loop {
        std::thread::park();
    }

}
