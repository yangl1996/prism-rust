use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::visualization;
use prism::{self, blockchain, blockdb, miner::memory_pool};
use std::sync::{Arc, Mutex};

const NUM_VOTER_CHAINS: u16 = 3;

fn main() {
    let exit_flag = Arc::new(Mutex::new(false));
    let exit_flag_dup = Arc::clone(&exit_flag);
    ctrlc::set_handler(move || {
        println!("Exiting");
        let mut ef = exit_flag.lock().unwrap();
        *ef = true;
    }).expect("Error setting Ctrl-C handler");
    stderrlog::new().verbosity(0).init().unwrap();

    // initialize all sorts of stuff for server 1
    let blockdb_path = std::path::Path::new("/tmp/prism_itest_two_node_mining_1.rocksdb");
    let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = Arc::new(blockdb);

    let blockchain = blockchain::BlockChain::new(NUM_VOTER_CHAINS);
    let blockchain = Arc::new(Mutex::new(blockchain));

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let peer_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let peer_port = 12345;
    let peer_addr_1 = std::net::SocketAddr::new(peer_ip, peer_port);

    let (_server_1, miner_1, _wallet_1) =
        prism::start(peer_addr_1, &blockdb, &blockchain, &mempool).unwrap();
    println!("Node 1 live at localhost:12345");

    let vis_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let vis_port = 8888;
    let vis_addr = std::net::SocketAddr::new(vis_ip, vis_port);
    visualization::Server::start(vis_addr, Arc::clone(&blockchain));
    println!("Node 1 visualization live at localhost:8888");

    // initialize all sorts of stuff for server 2
    let blockdb_path = std::path::Path::new("/tmp/prism_itest_two_node_mining_2.rocksdb");
    let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = Arc::new(blockdb);

    let blockchain = blockchain::BlockChain::new(NUM_VOTER_CHAINS);
    let blockchain = Arc::new(Mutex::new(blockchain));

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let peer_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let peer_port = 12346;
    let peer_addr = std::net::SocketAddr::new(peer_ip, peer_port);

    let (server_2, miner_2, _wallet_2) =
        prism::start(peer_addr, &blockdb, &blockchain, &mempool).unwrap();
    println!("Node 2 live at localhost:12346");
    server_2.connect(peer_addr_1);
    println!("Node connected to Node 1");

    let vis_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let vis_port = 8889;
    let vis_addr = std::net::SocketAddr::new(vis_ip, vis_port);
    visualization::Server::start(vis_addr, Arc::clone(&blockchain));
    println!("Node 2 visualization live at localhost:8889");

    // mine a block
    loop {
        if *exit_flag_dup.lock().unwrap() {
            break;
        }
        miner_1.step();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        if *exit_flag_dup.lock().unwrap() {
            break;
        }
        miner_2.step();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));
    miner_1.exit();
    miner_2.exit();
}
