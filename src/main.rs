#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate clap;

use log::{debug, error, info, warn};
use std::net;
use std::process;
use std::sync::Arc;

use prism::network;
use prism::blockdb;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_P2P_PORT: u16 = 6000;
const DEFAULT_BLOCKDB: &str = "/tmp/prismblocks.rocksdb";

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
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // init database
    let blockdb_path = match matches.value_of("block_db") {
        Some(path) => std::path::Path::new(path),
        None => std::path::Path::new(&DEFAULT_BLOCKDB),
    };
    let blockdb = blockdb::BlockDatabase::new(blockdb_path);

    // start p2p server
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

    debug!("Starting P2P server at {}", peer_socket_addr);
    let server = network::start(peer_socket_addr).unwrap();

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        for peer in known_peers {
            let addr = match peer.parse::<net::SocketAddr>() {
                Ok(x) => x,
                Err(e) => {
                    error!("Error parsing peer address {}: {}", &peer, e);
                    continue;
                },
            };
            match server.connect(addr) {
                Ok(_) => info!("Connected to outgoing peer {}", &addr),
                Err(e) => error!("Error connecting to peer {}: {}", addr, e),
            }
        }
    }

    loop {
        std::thread::park();
    }
}
