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
use rand::Rng;

#[test]
fn use_block_test() {
    use prism::block::tests::{proposer_block, voter_block, transaction_block};
    println!("{:?}", proposer_block([3u8;32].into(), 12345,vec![[4u8;32].into()],vec![]));
    println!("{:?}", voter_block([3u8;32].into(), 12345, 2, [9u8;32].into(), vec![[4u8;32].into()]));
    println!("{:?}", transaction_block([3u8;32].into(), 12345,vec![]));

}
//#[test]
//fn test_macro() {
//    macro_rules! random_extra_content {
//        () => {{
//            let mut rng = rand::thread_rng();
//            let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen_range(0, 255) as u8).collect();
//            let mut raw_bytes = [0; 32];
//            raw_bytes.copy_from_slice(&random_bytes);
//            raw_bytes
//        }};
//    }
//    macro_rules! random_nonce {
//        () => {{
//            let mut rng = rand::thread_rng();
//            let random_u32: u32 = rng.gen();
//            random_u32
//        }};
//    }
//    macro_rules! proposer_block {
//        ( $parent_hash:expr, $timestamp:expr, $proposer_refs:expr, $transaction_refs:expr ) => {{
//                Block::new($parent_hash, $timestamp as u64, random_nonce!(), [0u8;32].into(), vec![],
//                   Content::Proposer(proposer::Content {
//                       transaction_refs: $transaction_refs,
//                       proposer_refs: $proposer_refs,
//                   }), [0u8;32], *config::DEFAULT_DIFFICULTY)
//        }};
//    }
//    macro_rules! voter_block {
//        ( $parent_hash:expr, $timestamp:expr, $chain_num:expr, $voter_parent:expr, $votes:expr ) => {{
//                Block::new($parent_hash, $timestamp as u64, random_nonce!(), [0u8;32].into(), vec![],
//                   Content::Voter(voter::Content {
//                       chain_number: $chain_num,
//                       voter_parent: $voter_parent,
//                       votes: $votes,
//                   }), [0u8;32], *config::DEFAULT_DIFFICULTY)
//        }};
//    }
//    macro_rules! transaction_block {
//        ( $parent_hash:expr, $timestamp:expr, $transactions:expr ) => {{
//                Block::new($parent_hash, $timestamp as u64, random_nonce!(), [0u8;32].into(), vec![],
//                   Content::Transaction(transaction::Content {
//                       transactions: $transactions,
//                   }), [0u8;32], *config::DEFAULT_DIFFICULTY)
//        }};
//    }
//}
//#[test]
//fn integration() {
//
//    let blockdb = BlockDatabase::new("/tmp/prism_test_integration_blockdb.rocksdb").unwrap();
//
//    let blockchain = BlockChain::new("/tmp/prism_test_integration_blockchain.rocksdb").unwrap();
//
//    let utxodb = UtxoDatabase::new("/tmp/prism_test_integration_utxodb.rocksdb").unwrap();
//
//    let wallet = Wallet::new("/tmp/prism_test_integration_walletdb.rocksdb").unwrap();
//
//    let mempool = Mutex::new(MemoryPool::new());
//
//    let (msg_tx, _msg_rx) = mpsc::channel();
//    let (_ctx, server) = server::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10999),msg_tx).expect("fail at creating server");
//    let mut timestamp: u64 = 0;
//    {
//        let parent = blockchain.best_proposer();
//        let block = Block::new(parent, timestamp as u64, 0, [0u8;32].into(), vec![],
//                   Content::Proposer(proposer::Content {
//                       transaction_refs: vec![],
//                       proposer_refs: vec![],
//                   }), [0u8;32], *config::DEFAULT_DIFFICULTY);
//        new_validated_block(&block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
//    }
//    timestamp += 1;
//    for chain_num in 0..config::NUM_VOTER_CHAINS {
//        let parent = blockchain.best_proposer();
//        let voter_parent = blockchain.best_voter(chain_num as usize);
//        let block = Block::new(parent, timestamp as u64, 0, [0u8;32].into(), vec![],
//                               Content::Voter(voter::Content {
//                                   chain_number: chain_num,
//                                   voter_parent,
//                                   votes: vec![],
//                               }), [0u8;32], *config::DEFAULT_DIFFICULTY);
//        new_validated_block(&block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
//    }
//    timestamp += 1;
//    {
//        let parent = blockchain.best_proposer();
//        let block = Block::new(parent, timestamp as u64, 0, [0u8;32].into(), vec![],
//                               Content::Transaction(transaction::Content {
//                                   transactions: vec![],
//                               }), [0u8;32], *config::DEFAULT_DIFFICULTY);
//        new_validated_block(&block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
//    }
//}