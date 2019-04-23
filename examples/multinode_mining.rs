use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::visualization;
use prism::{self, blockchain, blockdb, state, miner::memory_pool};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::process;

const NUM_VOTER_CHAINS: u16 = 3;

fn main() {
    // clean up databases
    process::Command::new("sh")
        .arg("-c")
        .arg("rm")
        .arg("-rf")
        .arg("/tmp/prism_multinode_mining*")
        .output()
        .expect("Failed to clean up previous databases");
    let exit_flag = Arc::new(Mutex::new(false));
    let exit_flag_dup = Arc::clone(&exit_flag);
    ctrlc::set_handler(move || {
        println!("Exiting");
        let mut ef = exit_flag.lock().unwrap();
        *ef = true;
    })
    .expect("Error setting Ctrl-C handler");
    stderrlog::new().verbosity(3).init().unwrap();

    let mut peer_addrs = vec![];
    let mut servers = vec![];
    let mut miners = vec![];
    let mut channels = vec![];

    for i in 0..10 {
        let blockdb_path_string = format!("/tmp/prism_multinode_mining_{}_blocks.rocksdb", i);
        let blockdb_path = std::path::Path::new(&blockdb_path_string);
        let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
        let blockdb = Arc::new(blockdb);

        let utxodb_path_string = format!("/tmp/prism_multinode_mining_{}_utxo.rocksdb", i);
        let utxo_path = std::path::Path::new(&utxodb_path_string);
        let utxodb = state::UTXODatabase::new(utxo_path).unwrap();
        let utxodb = Arc::new(utxodb);

        let (state_update_sink, state_update_source) = mpsc::channel();
        let blockchain = blockchain::BlockChain::new(NUM_VOTER_CHAINS, state_update_sink);
        let blockchain = Arc::new(Mutex::new(blockchain));
        channels.push(state_update_source);

        let mempool = memory_pool::MemoryPool::new();
        let mempool = Arc::new(Mutex::new(mempool));

        let peer_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
        let peer_port = 10000 + i;
        let peer_addr = std::net::SocketAddr::new(peer_ip, peer_port);

        let (server, miner, _wallet) =
            prism::start(peer_addr, &blockdb, &utxodb, &blockchain, &mempool).unwrap();
        println!("Node {} live at localhost:{}", i, peer_port);

        let vis_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
        let vis_port = 8000 + i;
        let vis_addr = std::net::SocketAddr::new(vis_ip, vis_port);
        visualization::Server::start(vis_addr, Arc::clone(&blockchain));
        println!("Node {} visualization live at localhost:{}", i, vis_port);

        peer_addrs.push(peer_addr);
        servers.push(server);
        miners.push(miner);
    }

    for i in 1..10 {
        servers[i].connect(peer_addrs[i - 1]).unwrap();
        println!("Node {} connected to Node {}", i, i - 1);
    }

    // mine a block
    loop {
        for i in 0..10 {
            if *exit_flag_dup.lock().unwrap() {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                for i in 0..10 {
                    miners[i].exit();
                }
                return;
            }
            miners[i].step();
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}
