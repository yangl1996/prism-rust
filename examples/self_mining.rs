use prism::crypto::hash::{Hashable, H256};
use prism::transaction::{Output, Transaction};
use prism::visualization;
use prism::{self, blockchain, blockdb, miner::memory_pool};
use std::sync::{Arc, Mutex};

const NUM_VOTER_CHAINS: u16 = 3;

fn main() {
    // initialize all sorts of stuff
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

    let (_server, miner, mut wallet) =
        prism::start(peer_addr, &blockdb, &blockchain, &mempool).unwrap();

    let vis_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let vis_port = 8888;
    let vis_addr = std::net::SocketAddr::new(vis_ip, vis_port);
    visualization::Server::start(vis_addr, Arc::clone(&blockchain));

    // get an address from the wallet
    wallet.generate_keypair();//add_keypair(our_addr);
    let our_addr: H256 = wallet.get_pubkey_hash().unwrap();

    // fund-raising
    let funding = Transaction {
        input: vec![],
        output: vec![Output {
            value: 1000000,
            recipient: our_addr,
        }],
        signatures: vec![],
    };
    wallet.receive(&funding);
    assert_eq!(wallet.balance(), 1000000);

    // send some money to outself
    assert!(wallet.pay(our_addr, 5000).is_ok());
    // the transaction has not been mined, so our balance will dip for now
    assert_eq!(wallet.balance(), 0);

    // mine a block
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    miner.step();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    miner.exit();

    loop {
        std::thread::park();
    }
}
