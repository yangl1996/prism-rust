use prism::{network, blockdb, blockchain, miner::memory_pool, self};
use std::sync::{Arc, Mutex};

const NUM_VOTER_CHAINS: u16 = 3;

#[test]
fn self_mining() {
    let blockdb_path = std::path::Path::new("/tmp/prism_itest_self_mining.rocksdb");
    let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = Arc::new(blockdb);

    let blockchain = blockchain::BlockChain::new(NUM_VOTER_CHAINS);
    let blockchain = Arc::new(Mutex::new(blockchain));

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let peer_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let peer_port = 12345;
    let peer_addr = std::net::SocketAddr::new(peer_ip, peer_port);

    let (server, miner, wallet) = prism::start(peer_addr, &blockdb, &blockchain, &mempool).unwrap();
    miner.exit();
}
