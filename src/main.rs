#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate clap;

use log::{debug, info, warn, error};
use std::process;
use std::net;

mod hash;
mod merkle;
mod network;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_P2P_PORT: u16 = 6000;

fn main() {
    // parse command line arguments
    let matches = clap_app!(Prism =>
    (version: "0.1")
    (about: "Prism blockchain full client")
    (@arg verbose: -v ... "Increases the verbosity of logging")
    (@arg peer_ip: --ip [IP] "Sets the IP address to listen to peers")
    (@arg peer_port: --port [PORT] "Sets the port to listen to peers")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // start p2p server
    let peer_ip = match matches.value_of("peer_ip") {
        Some(ip) => {
            match ip.parse::<net::IpAddr>() {
                Ok(parsed) => parsed,
                Err(e) => {
                    error!("Error parsing IP address: {}", e);
                    process::exit(1);
                },
            }
        },
        None => DEFAULT_IP.parse::<net::IpAddr>().unwrap(),
    };
    let peer_port = match matches.value_of("peer_port") {
        Some(port) => {
            match port.parse::<u16>() {
                Ok(parsed) => parsed,
                Err(e) => {
                    error!("Error parsing p2p port: {}", e);
                    process::exit(1);
                },
            }
        },
        None => DEFAULT_P2P_PORT,
    };
    let peer_socket_addr = net::SocketAddr::new(peer_ip, peer_port);
    debug!("Starting P2P server at {}", peer_socket_addr);

    network::server::p2p_server(peer_socket_addr);
    loop {
    }
}
