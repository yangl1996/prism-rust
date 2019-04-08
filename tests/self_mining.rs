use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::{self, blockchain, blockdb, miner::memory_pool};
use std::sync::{Arc, Mutex};
use prism::visualization;

const NUM_VOTER_CHAINS: u16 = 3;

#[test]
fn self_mining() {
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

    // insert a fake key into the wallet
    let our_addr: H256 = (&[0; 32]).into();
    wallet.add_key(our_addr);

    // fund-raising
    let funding = Transaction {
        input: vec![],
        output: vec![Output {
            value: 1000000,
            recipient: our_addr,
        }],
        signatures: vec![],
    };
    wallet.add_transaction(&funding);
    assert_eq!(wallet.balance(), 1000000);

    // send some money to outself
    wallet.send_coin(our_addr, 5000);
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
    println!("{}", visualization::dump_blockchain(&blockchain.lock().unwrap()));
    miner.exit();

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
