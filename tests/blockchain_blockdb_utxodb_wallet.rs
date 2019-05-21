use prism::blockchain::BlockChain;
use prism::wallet::Wallet;
use prism::blockdb::BlockDatabase;
use prism::utxodb::UtxoDatabase;
use prism::block::{Block, Content, proposer, voter, transaction};
use prism::crypto::hash::Hashable;
use prism::transaction::Transaction;
use prism::miner::memory_pool::MemoryPool;
use prism::handler::new_validated_block;
use prism::network::server;
use prism::config;
use std::sync::{Mutex, mpsc};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use prism::block::tests::{proposer_block, voter_block, transaction_block};

#[test]
fn integration() {

    // this section we define the timestamp and macros that increment timestamp automatically
    let mut timestamp: u64 = 0;
    macro_rules! proposer_block {
        ( $parent_hash:expr, $proposer_refs:expr, $transaction_refs:expr ) => {{
            timestamp += 1;
            proposer_block($parent_hash, timestamp, $proposer_refs, $transaction_refs)
        }};
        ( $parent_hash:expr ) => {{
            timestamp += 1;
            proposer_block($parent_hash, timestamp, vec![], vec![])
        }};
    }
    macro_rules! voter_block {
        ( $parent_hash:expr, $chain_number:expr, $voter_parent:expr, $votes:expr ) => {{
            timestamp += 1;
            voter_block($parent_hash, timestamp, $chain_number, $voter_parent, $votes)
        }};
    }
    macro_rules! transaction_block {
        ( $parent_hash:expr, $transactions:expr ) => {{
            timestamp += 1;
            transaction_block( $ parent_hash, timestamp, $ transactions)
        }};
    }

    // create the db and ds
    let blockdb = BlockDatabase::new("/tmp/prism_test_integration_blockdb.rocksdb").unwrap();

    let blockchain = BlockChain::new("/tmp/prism_test_integration_blockchain.rocksdb").unwrap();

    let utxodb = UtxoDatabase::new("/tmp/prism_test_integration_utxodb.rocksdb").unwrap();

    let wallet = Wallet::new("/tmp/prism_test_integration_walletdb.rocksdb").unwrap();

    let mempool = Mutex::new(MemoryPool::new());

    let (msg_tx, _msg_rx) = mpsc::channel();
    let (_ctx, server) = server::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10999),msg_tx).expect("fail at creating server");

    let mut parent = blockchain.best_proposer();
    new_validated_block(&proposer_block!(parent),
                        &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);

}