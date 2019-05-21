use prism::blockchain::BlockChain;
use prism::wallet::Wallet;
use prism::blockdb::BlockDatabase;
use prism::utxodb::UtxoDatabase;
use prism::crypto::hash::Hashable;
use prism::transaction::{Transaction, tests as tx_generator, CoinId, Input, Output};
use prism::miner::memory_pool::MemoryPool;
use prism::handler::new_validated_block;
use prism::network::server;
use prism::config;
use std::sync::{Mutex, mpsc};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use prism::block::tests::{proposer_block, voter_block, transaction_block};
use prism::block::Content;

#[test]
fn integration() {
    // create the db and ds
    let blockdb = BlockDatabase::new("/tmp/prism_test_integration_blockdb.rocksdb").unwrap();

    let blockchain = BlockChain::new("/tmp/prism_test_integration_blockchain.rocksdb").unwrap();

    let utxodb = UtxoDatabase::new("/tmp/prism_test_integration_utxodb.rocksdb").unwrap();

    let wallet = Wallet::new("/tmp/prism_test_integration_walletdb.rocksdb").unwrap();
    wallet.generate_keypair().unwrap();
    let wallet_address = wallet.get_an_address().unwrap();
    let mempool = Mutex::new(MemoryPool::new());

    let (msg_tx, _msg_rx) = mpsc::channel();
    let (_ctx, server) = server::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10999),msg_tx).expect("fail at creating server");

    // this section we define the timestamp and macros that increment timestamp automatically
    let mut timestamp: u64 = 0;
    let mut parent_hash = blockchain.best_proposer();
    macro_rules! proposer_block {
        ( $proposer_refs:expr, $transaction_refs:expr ) => {{
            timestamp += 1;
            proposer_block(parent_hash, timestamp, $proposer_refs, $transaction_refs)
        }};
        () => {{
            timestamp += 1;
            proposer_block(parent_hash, timestamp, vec![], vec![])
        }};
    }
    macro_rules! voter_block {
        ( $chain_number:expr, $voter_parent:expr, $votes:expr ) => {{
            timestamp += 1;
            voter_block(parent_hash, timestamp, $chain_number, $voter_parent, $votes)
        }};
    }
    macro_rules! transaction_block {
        ( $transactions:expr ) => {{
            timestamp += 1;
            transaction_block(parent_hash, timestamp, $transactions)
        }};
    }
    macro_rules! handle_block {
        ( $block:expr ) => {{
            new_validated_block(&$block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
        }};
    }
    macro_rules! unwrap_transaction {
        ( $block:expr ) => {{
            if let Content::Transaction(c) = &$block.content {
                c.transactions.iter()
            } else {
                [].iter()
            }
        }};
    }

    // start test
    assert_eq!(blockchain.unreferred_transaction().len(),0);
    assert_eq!(blockchain.unreferred_proposer().len(),0);
    assert_eq!(wallet.balance().unwrap(), 0);

    let transaction_1 = transaction_block!(
            (0..3).map(|_|tx_generator::generate_random_transaction()).collect());
    let transaction_2 = transaction_block!(
            (0..5).map(|_|tx_generator::generate_random_transaction()).collect());
    handle_block!(transaction_1);
    handle_block!(transaction_2);
    assert_eq!(blockchain.unreferred_transaction().len(),2);
    assert!(blockchain.unreferred_transaction().contains(&transaction_1.hash()));
    assert!(blockchain.unreferred_transaction().contains(&transaction_2.hash()));

    //this proposer refers transaction blocks, and is to be referred by someone
    let proposer_1 = proposer_block!(vec![],vec![transaction_1.hash()]);
    handle_block!(proposer_1);
    assert_eq!(blockchain.unreferred_transaction().len(),1);
    assert_eq!(blockchain.unreferred_proposer().len(),1);
    assert!(blockchain.unreferred_proposer().contains(&proposer_1.hash()));

    //this proposer refers previous proposer and transaction blocks
    let proposer_2 = proposer_block!(vec![proposer_1.hash()], vec![transaction_2.hash()]);
    handle_block!(proposer_2);
    assert_eq!(blockchain.unreferred_transaction().len(),0);
    assert_eq!(blockchain.unreferred_proposer().len(),1);
    assert!(blockchain.unreferred_proposer().contains(&proposer_2.hash()));

    //change the proposer parent
    parent_hash = proposer_2.hash();

    //voters vote for proposer_2, and it becomes new leader
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_2.hash()]);
        handle_block!(v);
    }
    for t in unwrap_transaction!(transaction_1) {
        let hash = t.hash();
        for index in 0..t.output.len() {
            assert!(utxodb.contains(&CoinId{hash, index: index as u32}).unwrap());
        }
    }
    for t in unwrap_transaction!(transaction_2) {
        let hash = t.hash();
        for index in 0..t.output.len() {
            assert!(utxodb.contains(&CoinId{hash, index: index as u32}).unwrap());
        }
    }

    //grow the proposer tree and add transaction blocks
    let transaction_3 = transaction_block!(
            (0..2).map(|_|tx_generator::generate_random_transaction()).collect());
    handle_block!(transaction_3);
    let proposer_3 = proposer_block!(vec![], vec![transaction_3.hash()]);
    handle_block!(proposer_3);

    parent_hash = proposer_3.hash();
    // this transaction block spends the tokens in transaction_2, and transfers them to wallet
    let mut value_4 = 0u64;
    let transaction_4 = transaction_block!({
        let mut txs = vec![];
        for t in unwrap_transaction!(transaction_2) {
            let hash = t.hash();
            let mut input = vec![];
            let mut value = 0u64;
            for index in 0..t.output.len() {
                value += t.output[index].value;
                input.push(Input{
                    coin: CoinId{hash, index: index as u32},
                    value: t.output[index].value,
                    owner: t.output[index].recipient,
                })
            }
            txs.push(Transaction{
                input,
                output: vec![Output{
                    value,
                    recipient: wallet_address,
                }],
                authorization: vec![],
            });
            value_4 += value;
        }
        txs
    });
    handle_block!(transaction_4);
    let proposer_4 = proposer_block!(vec![], vec![transaction_4.hash()]);
    handle_block!(proposer_4);
    parent_hash = proposer_4.hash();

    //voters vote for proposer_3 and 4
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_4.hash(), proposer_3.hash()]);
        handle_block!(v);
    }
    for t in unwrap_transaction!(transaction_3) {
        let hash = t.hash();
        for index in 0..t.output.len() {
            assert!(utxodb.contains(&CoinId{hash, index: index as u32}).unwrap());
        }
    }
    for t in unwrap_transaction!(transaction_4) {
        let hash = t.hash();
        for index in 0..t.output.len() {
            assert!(utxodb.contains(&CoinId{hash, index: index as u32}).unwrap());
        }
    }
    for t in unwrap_transaction!(transaction_2) {
        let hash = t.hash();
        for index in 0..t.output.len() {
            assert!(!utxodb.contains(&CoinId{hash, index: index as u32}).unwrap());
        }
    }
    assert_eq!(wallet.balance().unwrap(), value_4);
//    utxodb.contains()
//    assert_eq!(blockchain.unreferred_transaction().len(),0);
//    assert_eq!(blockchain.unreferred_proposer().len(),0);
}