use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::visualization;
use prism::{self, blockchain, blockdb, miner::memory_pool, state, handler};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use prism::blockchain::transaction::UpdateMessage as LedgerUpdateMessage;
use std::thread;

const NUM_VOTER_CHAINS: u16 = 3;

fn main() {
    // initialize all sorts of stuff
    let blockdb_path = std::path::Path::new("/tmp/prism_itest_self_mining.rocksdb");
    let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = Arc::new(blockdb);

    let utxodb_path = std::path::Path::new("/tmp/prism_test_state.rocksdb");
    let utxodb = state::UTXODatabase::new(utxodb_path).unwrap();
    let utxodb = Arc::new(Mutex::new(utxodb));

    let (state_update_sink, state_update_source) = mpsc::channel();
    let blockchain = blockchain::BlockChain::new(NUM_VOTER_CHAINS, state_update_sink);
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

    wallet.generate_keypair();
    // insert a fake key into the wallet
    let our_addr: H256 = wallet.get_pubkey_hash().unwrap();
    let wallets = vec![Arc::new(Mutex::new(wallet))];
    // fund-raising
    let funding = Transaction {
        input: vec![],
        output: (0..1).map(|_| Output {
                    value: 100,
                    recipient: our_addr.clone(),
                }).collect(),
        key_sig: vec![],
    };
    handler::new_transaction(funding, &mempool);

    // mine blocks
    for _ in 0..50 {
        miner.step();
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));
    //miner.exit();
    //use thread::spawn(move || {
    loop {
        // get the new control singal from the channel
        match state_update_source.try_recv() {
            Ok((signal, hashes)) => {
                match (signal, hashes) {
                    (LedgerUpdateMessage::Add, hashes) => {
                        println!("Ledger add {:?}", hashes);
                        handler::confirm_new_tx_block_hashes(hashes, &blockdb, &utxodb, &wallets);
                        //, unconfirm_old_tx_block_transactions};

                    },
                    (LedgerUpdateMessage::Rollback, hashes) => {
                        println!("Ledger rollback {:?}", hashes);
                        handler::unconfirm_old_tx_block_hashes(hashes, &blockdb, &utxodb, &wallets);
                    },
                }
                println!(
                    "Balance of wallets: {:?}.",
                    wallets
                        .iter()
                        .map(|w| w.lock().unwrap().balance())
                        .collect::<Vec<u64>>()
                );
            }
            Err(_) => {println!("state update finished.");break;}
        }
    }
    {
        wallets.iter().for_each(|w|{
            match w.lock().unwrap().pay((&[0u8;32]).into(), 1) {
                Ok(hash) => (),
                Err(_) => (),
            }
        });
    }
    // mine blocks
    for _ in 0..50 {
        miner.step();
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));
    //miner.exit();

    loop {
        // get the new control singal from the channel
        match state_update_source.try_recv() {
            Ok((signal, hashes)) => {
                match (signal, hashes) {
                    (LedgerUpdateMessage::Add, hashes) => {
                        println!("Ledger add {:?}", hashes);
                        handler::confirm_new_tx_block_hashes(hashes, &blockdb, &utxodb, &wallets);
                        //, unconfirm_old_tx_block_transactions};

                    },
                    (LedgerUpdateMessage::Rollback, hashes) => {
                        println!("Ledger rollback {:?}", hashes);
                        handler::unconfirm_old_tx_block_hashes(hashes, &blockdb, &utxodb, &wallets);
                    },
                }
                println!(
                    "Balance of wallets: {:?}.",
                    wallets
                        .iter()
                        .map(|w| w.lock().unwrap().balance())
                        .collect::<Vec<u64>>()
                );
            }
            Err(_) => {println!("state update finished.");break;}
        }
    }

    loop {
        std::thread::park();
    }

}
