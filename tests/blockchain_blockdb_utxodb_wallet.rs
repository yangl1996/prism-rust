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
use prism::config::NUM_VOTER_CHAINS;
use std::sync::{Mutex, mpsc};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

#[test]
fn integration() {

    let blockdb = BlockDatabase::new("/tmp/prism_test_integration_blockdb.rocksdb").unwrap();

    let blockchain = BlockChain::new("/tmp/prism_test_integration_blockchain.rocksdb").unwrap();

    let utxodb = UtxoDatabase::new("/tmp/prism_test_integration_utxodb.rocksdb").unwrap();

    let wallet = Wallet::new("/tmp/prism_test_integration_walletdb.rocksdb").unwrap();

    let mempool = Mutex::new(MemoryPool::new());

    let (msg_tx, _msg_rx) = mpsc::channel();
    let (_ctx, server) = server::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10999),msg_tx).expect("fail at creating server");
    let mut timestamp: u64 = 0;
    {
        let parent = blockchain.best_proposer();
        let block = Block::new(parent, timestamp as u64, 0, [0u8;32].into(), vec![],
                   Content::Proposer(proposer::Content {
                       transaction_refs: vec![],
                       proposer_refs: vec![],
                   }), [0u8;32], [255u8;32].into());
        new_validated_block(&block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
    }
    timestamp += 1;
    for chain_num in 0..NUM_VOTER_CHAINS {
        let parent = blockchain.best_proposer();
        let voter_parent = blockchain.best_voter(chain_num as usize);
        let block = Block::new(parent, timestamp as u64, 0, [0u8;32].into(), vec![],
                               Content::Voter(voter::Content {
                                   chain_number: chain_num,
                                   voter_parent,
                                   votes: vec![],
                               }), [0u8;32], [255u8;32].into());
        new_validated_block(&block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
    }
    timestamp += 1;
    {
        let parent = blockchain.best_proposer();
        let block = Block::new(parent, timestamp as u64, 0, [0u8;32].into(), vec![],
                               Content::Transaction(transaction::Content {
                                   transactions: vec![],
                               }), [0u8;32], [255u8;32].into());
        new_validated_block(&block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
    }
}