#[macro_use]
extern crate clap;

use log::{debug, error, info};
use prism::blockchain::BlockChain;
use prism::blockdb::BlockDatabase;
use prism::miner::memory_pool::MemoryPool;
use prism::utxodb::UtxoDatabase;
use prism::visualization::Server as VisualizationServer;
use prism::wallet::Wallet;
use prism::api::Server as ApiServer;
use std::net;
use std::process;
use std::sync::Arc;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_P2P_PORT: u16 = 6000;
const DEFAULT_API_ADDR: &str = "127.0.0.1:7000";
const DEFAULT_BLOCKDB: &str = "/tmp/prism-blocks.rocksdb";
const DEFAULT_UTXODB: &str = "/tmp/prism-utxo.rocksdb";
const DEFAULT_BLOCKCHAIN: &str = "/tmp/prism-blockchain.rocksdb";
const DEFAULT_WALLET: &str = "/tmp/prism-wallet.rocksdb";

fn main() {
    // parse command line arguments
    let matches = clap_app!(Prism =>
     (version: "0.1")
     (about: "Prism blockchain full client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_ip: --ip [IP] "Sets the IP address to listen to peers")
     (@arg peer_port: --port [PORT] "Sets the port to listen to peers")
     (@arg visualization: --visual [ADDR] "Enables the visualization server at the given address and port")
     (@arg api: --api [ADDR] "Sets the IP address and the port of the API server")
     (@arg mine: -m --mine "Enables the CPU miner to mine blocks")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to")
     (@arg block_db: --blockdb [PATH] "Sets the path of the block database")
     (@arg utxo_db: --utxodb [PATH] "Sets the path of the UTXO database")
     (@arg blockchain_db: --blockchaindb [PATH] "Sets the path of the blockchain database")
     (@arg wallet_db: --walletdb [PATH] "Sets the path of the wallet")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // init mempool
    let mempool = MemoryPool::new();
    let mempool = Arc::new(std::sync::Mutex::new(mempool));

    // init block database
    let blockdb = match matches.value_of("block_db") {
        Some(path) => BlockDatabase::new(&path).unwrap(),
        None => BlockDatabase::new(&DEFAULT_BLOCKDB).unwrap(),
    };
    let blockdb = Arc::new(blockdb);

    // init utxo database
    let utxodb = match matches.value_of("utxo_db") {
        Some(path) => UtxoDatabase::new(&path).unwrap(),
        None => UtxoDatabase::new(&DEFAULT_UTXODB).unwrap(),
    };
    let utxodb = Arc::new(utxodb);

    // init blockchain database
    let blockchain = match matches.value_of("blockchain_db") {
        Some(path) => BlockChain::new(&path).unwrap(),
        None => BlockChain::new(&DEFAULT_BLOCKCHAIN).unwrap(),
    };
    let blockchain = Arc::new(blockchain);

    // init wallet database
    let wallet = match matches.value_of("wallet_db") {
        Some(path) => Wallet::new(&path).unwrap(),
        None => Wallet::new(&DEFAULT_WALLET).unwrap(),
    };
    let wallet = Arc::new(wallet);

    // start visualization server
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

    // parse server ip and port
    let peer_ip = match matches.value_of("peer_ip") {
        Some(ip) => ip.parse::<net::IpAddr>().unwrap_or_else(|e| {
            error!("Error parsing P2P IP address: {}", e);
            process::exit(1);
        }),
        None => DEFAULT_IP.parse::<net::IpAddr>().unwrap(),
    };
    let peer_port = match matches.value_of("peer_port") {
        Some(port) => port.parse::<u16>().unwrap_or_else(|e| {
            error!("Error parsing P2P port: {}", e);
            process::exit(1);
        }),
        None => DEFAULT_P2P_PORT,
    };
    let peer_socket_addr = net::SocketAddr::new(peer_ip, peer_port);

    // parse api server address
    let api_addr = match matches.value_of("api") {
        Some(addr) => addr.parse::<net::SocketAddr>().unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        }),
        None => DEFAULT_API_ADDR.parse::<net::SocketAddr>().unwrap()
    };

    // init server and miner
    let (server, miner) = prism::start(
        peer_socket_addr,
        &blockdb,
        &utxodb,
        &blockchain,
        &wallet,
        &mempool,
    )
    .unwrap();

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        for peer in known_peers {
            let addr = match peer.parse::<net::SocketAddr>() {
                Ok(x) => x,
                Err(e) => {
                    error!("Error parsing peer address {}: {}", &peer, e);
                    continue;
                }
            };
            match server.connect(addr) {
                Ok(_) => info!("Connected to outgoing peer {}", &addr),
                Err(e) => error!("Error connecting to peer {}: {}", addr, e),
            }
        }
    }

    // start the miner
    if matches.is_present("mine") {
        miner.start();
    }

    // start the API server
    ApiServer::start(api_addr, &wallet, &server, &mempool);

    loop {
        std::thread::park();
    }
}
