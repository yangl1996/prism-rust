#[macro_use]
extern crate clap;

use log::{error, info};
use prism::blockchain::BlockChain;
use prism::blockdb::BlockDatabase;
use prism::miner::memory_pool::MemoryPool;
use prism::utxodb::UtxoDatabase;
use prism::visualization::Server as VisualizationServer;
use prism::wallet::Wallet;
use prism::api::Server as ApiServer;
use prism::network::server;
use prism::network::worker;
use prism::experiment::transaction_generator::TransactionGenerator;
use prism::miner;
use prism::crypto::hash::{H256, Hashable};
use prism::handler::update_ledger;
use std::net;
use std::process;
use std::sync::Arc;
use std::sync::mpsc;
use std::convert::TryInto;
use std::thread;
use std::time;
use rand::Rng;
use rand::rngs::OsRng;
use ed25519_dalek::Keypair;
use ed25519_dalek::Signature;
use prism::transaction::Address;
use prism::visualization::demo;

fn main() {
    // parse command line arguments
    let matches = clap_app!(Prism =>
     (version: "0.1")
     (about: "Prism blockchain full client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_addr: --p2p [ADDR] default_value("127.0.0.1:6000") "Sets the IP address and the port of the P2P server")
     (@arg api_addr: --api [ADDR] default_value("127.0.0.1:7000") "Sets the IP address and the port of the API server")
     (@arg visualization: --visual [ADDR] "Enables the visualization server at the given address and port")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to")
     (@arg block_db: --blockdb [PATH] default_value("/tmp/prism-blocks.rocksdb") "Sets the path of the block database")
     (@arg utxo_db: --utxodb [PATH] default_value("/tmp/prism-utxo.rocksdb") "Sets the path of the UTXO database")
     (@arg blockchain_db: --blockchaindb [PATH] default_value("/tmp/prism-blockchain.rocksdb") "Sets the path of the blockchain database")
     (@arg wallet_db: --walletdb [PATH] default_value("/tmp/prism-wallet.rocksdb") "Sets the path of the wallet")
     (@arg init_fund_addr: --("fund-addr") ... [HASH] "Endows the given address an initial fund")
     (@arg init_fund_coins: --("fund-coins") [INT] default_value("50000") "Sets the number of coins of the initial fund for each peer")
     (@arg init_fund_value: --("fund-value") [INT] default_value("100") "Sets the value of each initial fund coin")
     (@arg load_key_path: --("load-key") ... [PATH] "Loads a key pair into the wallet from the given address")
     (@arg mempool_size: --("mempool-size") ... [SIZE] default_value("500000") "Sets the size limit of the memory pool")
     (@arg demo_addr: --demo [ADDR] default_value("ws://127.0.0.1:9000") "Sets the IP address and the port of the demo websocket to connect to")
     (@arg demo_transaction_ratio: --("demo-tran-ratio") [INT] default_value("1") "Sets the ratio of transaction blocks in demo")
     (@arg demo_voter_max: --("demo-vote-max") [INT] default_value("1000") "Sets the max voter chain to show in demo")
     (@subcommand keygen =>
      (about: "Generates Prism wallet key pair")
      (@arg display_address: --addr "Prints the address of the key pair to STDERR")
     )
    )
    .get_matches();

    // match subcommands
    match matches.subcommand() {
        ("keygen", Some(m)) => {
            let mut csprng: OsRng = OsRng::new().unwrap();
            let keypair: Keypair = Keypair::generate(&mut csprng);
            let base64_encoded = base64::encode(&keypair.to_bytes().to_vec());
            println!("{}", base64_encoded);
            if m.is_present("display_address") {
                let addr: Address = ring::digest::digest(&ring::digest::SHA256, &keypair.public.as_bytes().as_ref()).into();
                let base64_encoded = base64::encode(&addr);
                eprintln!("{}", base64_encoded);
            }
            return;
        }
        _ => {}
    }

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // init mempool
    let mempool_size = matches.value_of("mempool_size").unwrap().parse::<u64>().unwrap_or_else(|e| {
        error!("Error parsing memory pool size limit: {}", e);
        process::exit(1);
    });
    let mempool = MemoryPool::new(mempool_size);
    let mempool = Arc::new(std::sync::Mutex::new(mempool));

    // init block database
    let blockdb = BlockDatabase::new(&matches.value_of("block_db").unwrap()).unwrap();
    let blockdb = Arc::new(blockdb);

    // init utxo database
    let utxodb = UtxoDatabase::new(&matches.value_of("utxo_db").unwrap()).unwrap();
    let utxodb = Arc::new(utxodb);

    // init blockchain database
    let blockchain = BlockChain::new(&matches.value_of("blockchain_db").unwrap()).unwrap();
    let blockchain = Arc::new(blockchain);

    // init wallet database
    let wallet = Wallet::new(&matches.value_of("wallet_db").unwrap()).unwrap();
    let wallet = Arc::new(wallet);

    // load wallet keys
    if let Some(wallet_keys) = matches.values_of("load_key_path") {
        for key_path in wallet_keys {
            let content = match std::fs::read_to_string(&key_path) {
                Ok(c) => c,
                Err(e) => {
                    error!("Error loading key pair at {}: {}", &key_path, &e);
                    process::exit(1);
                }
            };
            let decoded = match base64::decode(&content.trim()) {
                Ok(d) => d,
                Err(e) => {
                    error!("Error decoding key pair at {}: {}", &key_path, &e);
                    process::exit(1);
                }
            };
            let keypair = Keypair::from_bytes(&decoded).unwrap();
            match wallet.load_keypair(keypair) {
                Ok(a) => info!("Loaded key pair for address {}", &a),
                Err(e) => {
                    error!("Error loading key pair into wallet: {}", &e);
                    process::exit(1);
                }
            }
        }
    }

    // connect to demo websocket server
    let demo_transaction_ratio = matches.value_of("demo_transaction_ratio").unwrap().parse::<u32>().unwrap_or_else(|e| {
        error!("Error parsing number of demo_transaction_ratio: {}", e);
        process::exit(1);
    });
    let demo_voter_max = matches.value_of("demo_voter_max").unwrap().parse::<u16>().unwrap_or_else(|e| {
        error!("Error parsing number of demo_voter_max: {}", e);
        process::exit(1);
    });
    let demo_sender = demo::new(&matches.value_of("demo_addr").unwrap(), demo_transaction_ratio, demo_voter_max);

    // start thread to update ledger
    let blockdb_copy = Arc::clone(&blockdb);
    let blockchain_copy = Arc::clone(&blockchain);
    let utxodb_copy = Arc::clone(&utxodb);
    let wallet_copy = Arc::clone(&wallet);
    let demo_sender_copy = demo_sender.clone();
    let (tx_diff_tx, tx_diff_rx) = mpsc::sync_channel(3);
    let (coin_diff_tx, coin_diff_rx) = mpsc::sync_channel(3);
    thread::spawn(move || {
        loop {
            let tx_diff = update_ledger::update_transaction_sequence(&blockdb_copy, &blockchain_copy, &demo_sender_copy);
            tx_diff_tx.send(tx_diff).unwrap();
        }
    });
    thread::spawn(move || {
        loop {
            let tx_diff = tx_diff_rx.recv().unwrap();
            let coin_diff = update_ledger::update_utxo(&tx_diff.0, &tx_diff.1, &utxodb_copy);
            coin_diff_tx.send(coin_diff).unwrap();
        }
    });
    thread::spawn(move || {
        loop {
            let coin_diff = coin_diff_rx.recv().unwrap();
            update_ledger::update_wallet(&coin_diff.0, &coin_diff.1, &wallet_copy);
        }
    });

    // parse p2p server address
    let p2p_addr = matches.value_of("peer_addr").unwrap().parse::<net::SocketAddr>().unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
    });

    // parse api server address
    let api_addr = matches.value_of("api_addr").unwrap().parse::<net::SocketAddr>().unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
    });

    // create channels between server and worker, worker and miner, miner and worker
    let (msg_tx, msg_rx) = mpsc::channel();
    let (ctx_tx, ctx_rx) = mpsc::channel();

    // start the p2p server
    let (server_ctx, server) = server::new(p2p_addr, msg_tx).unwrap();
    server_ctx.start().unwrap();


    // start the worker
    let worker_ctx = worker::new(16, msg_rx, &blockchain, &blockdb, &utxodb, &wallet, &mempool, ctx_tx, &server, demo_sender.clone() );
    worker_ctx.start();

    // pass extra content to miner. extra content contains peer_addr which shows the node id
    let extra_content = {
        let mut bytes: [u8;32] = [0;32];
        let port: [u8;2] = p2p_addr.port().to_be_bytes();
        bytes[30..32].copy_from_slice(&port[..]);
        match p2p_addr.ip() {
            net::IpAddr::V4(v4) => {
                let v4: u32 = v4.into();
                let v4: [u8; 4] = v4.to_be_bytes();
                bytes[26..30].copy_from_slice(&v4[..]);
            }
            net::IpAddr::V6(v6) => {
                let v6: u128 = v6.into();
                let v6: [u8; 16] = v6.to_be_bytes();
                bytes[14..30].copy_from_slice(&v6[..]);
            }
        };
        bytes
    };
    // start the miner
    let (miner_ctx, miner) = miner::new(&mempool, &blockchain, &blockdb, ctx_rx, &server, extra_content, demo_sender.clone());
    miner_ctx.start();

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!("Error connecting to peer {}, retrying in one second: {}", addr, e);
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });
    }

    // fund the given addresses
    if let Some(fund_addrs) = matches.values_of("init_fund_addr") {
        let num_coins = matches.value_of("init_fund_coins").unwrap().parse::<usize>().unwrap_or_else(|e| {
            error!("Error parsing number of initial fund coins: {}", e);
            process::exit(1);
        });
        let coin_value = matches.value_of("init_fund_value").unwrap().parse::<u64>().unwrap_or_else(|e| {
            error!("Error parsing value of initial fund coins: {}", e);
            process::exit(1);
        });
        let mut addrs = vec![];
        for addr in fund_addrs {
            let decoded = match base64::decode(&addr.trim()) {
                Ok(d) => d,
                Err(e) => {
                    error!("Error decoding address {}: {}", &addr.trim(), e);
                    process::exit(1);
                }
            };
            let addr_bytes: [u8; 32] = (&decoded[0..32]).try_into().unwrap();
            let hash: H256 = addr_bytes.into();
            addrs.push(hash);
        }
        info!("Funding {} addresses with {} initial coins of {}", addrs.len(), num_coins, coin_value);
        prism::experiment::ico(&addrs, &utxodb, &wallet, num_coins, coin_value).unwrap();
    }

    // create wallet key pair if there is none
    if wallet.addresses().unwrap().len() == 0 {
        wallet.generate_keypair().unwrap();
    }

    // start the transaction generator
    let (txgen_ctx, txgen_control_chan) = TransactionGenerator::new(&wallet, &server, &mempool);
    txgen_ctx.start();

    // start the API server
    ApiServer::start(api_addr, &wallet, &blockchain, &utxodb, &server, &miner, &mempool, txgen_control_chan);

    // start the visualization server
    match matches.value_of("visualization") {
        Some(addr) => {
            let addr = addr.parse::<net::SocketAddr>().unwrap_or_else(|e| {
                error!("Error parsing visualization server socket address: {}", e);
                process::exit(1);
            });
            info!("Starting visualization server at {}", &addr);
            VisualizationServer::start(addr, &blockchain, &blockdb, &utxodb);
        }
        None => {}
    }

    loop {
        std::thread::park();
    }
}
