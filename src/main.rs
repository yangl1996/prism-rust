#[macro_use]
extern crate clap;

use log::{debug, error, info};
use prism::blockchain;
use prism::blockdb;
use prism::state;
use prism::config;
use prism::miner::memory_pool;
use std::net;
use std::process;
use std::sync::mpsc;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_P2P_PORT: u16 = 6000;
const DEFAULT_BLOCKDB: &str = "/tmp/prismblocks.rocksdb";
const DEFAULT_UTXODB: &str = "/tmp/prismutxo.rocksdb";

fn main() {
    // parse command line arguments
    let matches = clap_app!(Prism =>
     (version: "0.1")
     (about: "Prism blockchain full client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_ip: --ip [IP] "Sets the IP address to listen to peers")
     (@arg peer_port: --port [PORT] "Sets the port to listen to peers")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to")
     (@arg block_db: --blockdb ... [PATH] "Sets the path of the block database")
     (@arg utxo_db: --utxodb ... [PATH] "Sets the path of the UTXO database")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // init block database
    let blockdb_path = match matches.value_of("block_db") {
        Some(path) => std::path::Path::new(path),
        None => std::path::Path::new(&DEFAULT_BLOCKDB),
    };
    let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = std::sync::Arc::new(blockdb);

    // init utxo database
    let utxodb_path = match matches.value_of("utxo_db") {
        Some(path) => std::path::Path::new(path),
        None => std::path::Path::new(&DEFAULT_UTXODB),
    };
    let utxodb = state::UTXODatabase::new(utxodb_path).unwrap();
    let utxodb = std::sync::Arc::new(utxodb);

    // init blockchain
    let (state_update_sink, state_update_source) = mpsc::channel();
    let blockchain = blockchain::BlockChain::new(config::NUM_VOTER_CHAINS, state_update_sink);
    let blockchain = std::sync::Arc::new(std::sync::Mutex::new(blockchain));

    // init mempool
    let mempool = memory_pool::MemoryPool::new();
    let mempool = std::sync::Arc::new(std::sync::Mutex::new(mempool));

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

    // init server and miner
    debug!("Starting P2P server at {}", peer_socket_addr);
    let (server, miner, _wallet) =
        prism::start(peer_socket_addr, &blockdb, &utxodb, &blockchain, &mempool, state_update_source).unwrap();

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

    miner.start();

    loop {
        std::thread::park();
    }
}
