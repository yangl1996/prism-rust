use prism::blockchain::BlockChain;
use prism::wallet::Wallet;
use prism::blockdb::BlockDatabase;
use prism::utxodb::UtxoDatabase;
use prism::crypto::hash::{Hashable, H256};
use prism::transaction::{Transaction, tests as tx_generator, CoinId, Input, Output};
use prism::miner::memory_pool::MemoryPool;
use prism::handler::new_validated_block;
use prism::network::server;
use prism::config;
use std::sync::{Mutex, mpsc};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use prism::block::tests::{proposer_block, voter_block, transaction_block};
use prism::block::Content;
use prism::experiment::ico;

#[test]
fn integration() {
    // create the db and ds
    let blockdb = BlockDatabase::new("/tmp/prism_test_integration_blockdb.rocksdb").unwrap();

    let blockchain = BlockChain::new("/tmp/prism_test_integration_blockchain.rocksdb").unwrap();

    let utxodb = UtxoDatabase::new("/tmp/prism_test_integration_utxodb.rocksdb").unwrap();

    let wallet = Wallet::new("/tmp/prism_test_integration_walletdb.rocksdb").unwrap();
    wallet.generate_keypair().unwrap();
    let wallet_address = wallet.addresses().unwrap()[0];
    let mempool = Mutex::new(MemoryPool::new());

    let (msg_tx, _msg_rx) = mpsc::channel();
    let (_ctx, server) = server::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10999),msg_tx).expect("fail at creating server");

    // this section we define the timestamp and macros that increment timestamp automatically
    let mut timestamp: u128 = 0;
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
    // call new_validated_block 3 times to ensure robustness
    macro_rules! handle_block {
        ( $block:expr ) => {{
            new_validated_block(&$block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
            new_validated_block(&$block, &mempool, &blockdb, &blockchain, &server, &utxodb, &wallet);
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
    macro_rules! check_transaction_output {
        ( $block:expr, $bool:expr ) => {{
            for t in unwrap_transaction ! ($block) {
                let hash = t.hash();
                for index in 0..t.output.len() {
                    assert_eq ! (utxodb.contains( & CoinId{hash, index: index as u32}).unwrap(), $bool,
                    "for tx {:?} output index {}, error in utxo, should be {}",hash,index,$bool);
                }
            }
        }};
    }
    macro_rules! random_transaction_block_0_input {
        () => {{
            transaction_block!(
                vec![Transaction{input:vec![], output:(0..3).map(|_|tx_generator::generate_random_output()).collect(), authorization:vec![]}]
            )
        }};
    }

    // start test
    assert_eq!(blockchain.unreferred_transactions().len(),0);
    assert_eq!(blockchain.unreferred_proposers().len(),1);
    assert_eq!(wallet.balance().unwrap(), 0);

    //test ico
    ico(&[wallet_address], &utxodb, &wallet).unwrap();
    let ico_number = wallet.balance().unwrap();

    let transaction_1 = random_transaction_block_0_input!();
    let transaction_2 = random_transaction_block_0_input!();
    handle_block!(transaction_1);
    handle_block!(transaction_2);
    assert_eq!(blockchain.unreferred_transactions().len(),2);
    assert!(blockchain.unreferred_transactions().contains(&transaction_1.hash()));
    assert!(blockchain.unreferred_transactions().contains(&transaction_2.hash()));

    //this proposer refers transaction blocks, and is to be referred by someone
    let proposer_1 = proposer_block!(vec![],vec![transaction_1.hash()]);
    handle_block!(proposer_1);
    assert_eq!(blockchain.unreferred_transactions().len(),1);
    assert_eq!(blockchain.unreferred_proposers().len(),1);
    assert!(blockchain.unreferred_proposers().contains(&proposer_1.hash()));
    assert_eq!(proposer_1.hash(), blockchain.best_proposer());
    //create empty proposer (for proposer tree forking)
    let proposer_1_fork = proposer_block!();
    handle_block!(proposer_1_fork);
    assert_ne!(proposer_1_fork.hash(), blockchain.best_proposer());
    let proposer_1_fork_2 = proposer_block!();
    handle_block!(proposer_1_fork_2);

    parent_hash = proposer_1_fork.hash();
    //this proposer refers previous proposer and transaction blocks
    let proposer_2 = proposer_block!(vec![proposer_1.hash(), proposer_1_fork_2.hash()], vec![transaction_2.hash()]);
    handle_block!(proposer_2);
    assert_eq!(blockchain.unreferred_transactions().len(),0);
    assert_eq!(blockchain.unreferred_proposers().len(),1);
    assert!(blockchain.unreferred_proposers().contains(&proposer_2.hash()));
    assert_eq!(proposer_2.hash(), blockchain.best_proposer());
    //create empty proposer (for proposer tree forking)
    let proposer_2_fork = proposer_block!();
    handle_block!(proposer_2_fork);
    //change the proposer parent
    parent_hash = proposer_2.hash();

    //voters vote for proposer_2, and it becomes new leader
    let mut voter_parent_to_fork = vec![];
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_1.hash(), proposer_2.hash()]);
        handle_block!(v);
        voter_parent_to_fork.push(v);
    }

    check_transaction_output!(transaction_1, true);
    check_transaction_output!(transaction_2, true);

    //grow the proposer tree and add transaction blocks
    let transaction_3 = random_transaction_block_0_input!();
    handle_block!(transaction_3);
    let proposer_3 = proposer_block!(vec![proposer_2_fork.hash()], vec![transaction_3.hash()]);
    handle_block!(proposer_3);
    assert_eq!(proposer_3.hash(), blockchain.best_proposer());
    //create empty proposer (for proposer tree forking)
    //it's parent is proposer_1_fork_2
    parent_hash = proposer_2_fork.hash();
    let proposer_3_fork = proposer_block!();
    handle_block!(proposer_3_fork);

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
    let proposer_4_fork = proposer_block!(vec![proposer_3_fork.hash()], vec![transaction_4.hash()]);//refer tx 4 again
    handle_block!(proposer_4_fork);
    parent_hash = proposer_3_fork.hash();
    let proposer_4_fork_2 = proposer_block!();
    handle_block!(proposer_4_fork_2);
    parent_hash = proposer_4.hash();

    //voters vote for proposer_3 and 4
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_4.hash(), proposer_3.hash()]);
        handle_block!(v);
    }
    check_transaction_output!(transaction_3, true);
    check_transaction_output!(transaction_4, true);
    check_transaction_output!(transaction_2, false);

    assert_eq!(wallet.balance().unwrap(), value_4+ico_number);

    let transaction_5 = random_transaction_block_0_input!();
    let transaction_6 = random_transaction_block_0_input!();
    handle_block!(transaction_5);
    handle_block!(transaction_6);
    let proposer_5 = proposer_block!(vec![], vec![transaction_5.hash()]);
    parent_hash = proposer_5.hash();
    let proposer_6 = proposer_block!(vec![proposer_4_fork.hash(), proposer_4_fork_2.hash()], vec![transaction_6.hash()]);
    parent_hash = proposer_6.hash();
    handle_block!(proposer_5);
    handle_block!(proposer_6);
    //test proposer_6 is leader but proposer_5 is not, ledger should not grow
    //although this may fail validation
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_6.hash()]);
        handle_block!(v);
    }
    check_transaction_output!(transaction_5, false);
    check_transaction_output!(transaction_6, false);
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_5.hash()]);
        handle_block!(v);
    }
    check_transaction_output!(transaction_5, true);
    check_transaction_output!(transaction_6, true);

    let transaction_7 = random_transaction_block_0_input!();
    handle_block!(transaction_7);
    let proposer_7 = proposer_block!(vec![], vec![transaction_7.hash()]);
    handle_block!(proposer_7);
    parent_hash = proposer_7.hash();

    // the expression below depend on confirm algorithm
    let not_enough_vote = 1;//config::NUM_VOTER_CHAINS/2-1;
    for chain_number in 0..not_enough_vote {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_7.hash()]);
        handle_block!(v);
    }
    check_transaction_output!(transaction_7, false);

    for chain_number in not_enough_vote..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_7.hash()]);
        handle_block!(v);
    }
    check_transaction_output!(transaction_7, true);

    //test wallet create_transaction
    let receiver: H256 = [9u8;32].into();
    let payment = wallet.create_transaction(receiver, value_4-1).unwrap();
    let transaction_8 = transaction_block!(vec![payment.clone()]);
    handle_block!(transaction_8);
    let proposer_8 = proposer_block!(vec![], vec![transaction_8.hash()]);
    handle_block!(proposer_8);
    parent_hash = proposer_8.hash();
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![proposer_8.hash()]);
        handle_block!(v);
    }
    check_transaction_output!(transaction_8, true);
    assert_eq!(wallet.balance().unwrap(), 1+ico_number);

    // check proposers in ledger
    let ledger = blockchain.proposer_transaction_in_ledger(100).unwrap();
    let ledger_proposer: Vec<H256> = ledger.into_iter().map(|x|x.0).collect();
    assert_eq!(ledger_proposer, vec![
        *config::PROPOSER_GENESIS_HASH,
        proposer_1.hash(),
        proposer_1_fork.hash(), proposer_1_fork_2.hash(),
        proposer_2.hash(),
        proposer_2_fork.hash(),
        proposer_3.hash(),
        proposer_4.hash(),
        proposer_5.hash(),
        proposer_3_fork.hash(),
        proposer_4_fork.hash(), proposer_4_fork_2.hash(),
        proposer_6.hash(),
        proposer_7.hash(),
        proposer_8.hash(),
    ]);

    //forking on voter chains, but fork length is equal to main chain length, so nothing happens
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, voter_parent_to_fork[chain_number as usize].hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        //the fork is not the best
        assert_ne!(v.hash(), blockchain.best_voter(chain_number as usize));
    }
    check_transaction_output!(transaction_8, true);
    assert_eq!(wallet.balance().unwrap(), 1+ico_number);

    //longer forking on voter chains, rollback should happen, and history should be changed
    parent_hash = proposer_2.hash();
    let transaction_9 = random_transaction_block_0_input!();
    handle_block!(transaction_9);
    //this transaction is to test invalid (wrt utxo) transaction
    let transaction_invalid = transaction_block!(vec![tx_generator::generate_random_transaction()]);
    handle_block!(transaction_invalid);
    let proposer_9 = proposer_block!(vec![], vec![transaction_9.hash(),transaction_invalid.hash()]);
    handle_block!(proposer_9);
    parent_hash = proposer_9.hash();

    let mut voter_parent_to_fork_2 = vec![];

    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, voter_parent_to_fork[chain_number as usize].hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        voter_parent_to_fork_2.push(v.hash());
        let v = voter_block!(chain_number, v.hash(), vec![proposer_9.hash()]);
        handle_block!(v);
        //the fork becomes the best
        assert_eq!(v.hash(), blockchain.best_voter(chain_number as usize));
    }
    //check the result after rolling back
    assert_eq!(blockchain.unreferred_proposers().len(),2);
    //we have fork in proposer tree, and in one fork proposer_8, in another fork proposer_9, these are 2 unreferred_proposers
    assert!(blockchain.unreferred_proposers().contains(&proposer_9.hash()));
    assert!(blockchain.unreferred_proposers().contains(&proposer_8.hash()));
    assert_eq!(blockchain.unreferred_transactions().len(),0);
    //although proposer_9 is leader now, it is not best proposer
    assert_ne!(proposer_9.hash(), blockchain.best_proposer());
    //only the proposer fork with proposer_9 is in ledger
    check_transaction_output!(transaction_1, true);
    check_transaction_output!(transaction_2, true);
    check_transaction_output!(transaction_3, false);
    check_transaction_output!(transaction_4, false);
    check_transaction_output!(transaction_5, false);
    check_transaction_output!(transaction_6, false);
    check_transaction_output!(transaction_7, false);
    check_transaction_output!(transaction_8, false);
    check_transaction_output!(transaction_9, true);
    check_transaction_output!(transaction_invalid, false);
    //see whether wallet balance rollback
    assert_eq!(wallet.balance().unwrap(), ico_number);

    //on this fork (proposer_9) we grow the proposer tree and ledger
    let transaction_10 = random_transaction_block_0_input!();
    handle_block!(transaction_10);
    //we create proposers on the fork to a length, also refer proposer_8 on another fork
    let proposer_10 = proposer_block!();
    handle_block!(proposer_10);
    parent_hash = proposer_10.hash();
    let proposer_11 = proposer_block!();
    handle_block!(proposer_11);
    parent_hash = proposer_11.hash();
    let proposer_12 = proposer_block!();
    handle_block!(proposer_12);
    parent_hash = proposer_12.hash();
    let proposer_13 = proposer_block!(vec![proposer_4.hash(), proposer_6.hash()], vec![]);
    handle_block!(proposer_13);
    parent_hash = proposer_13.hash();
    let proposer_14 = proposer_block!();
    handle_block!(proposer_14);
    parent_hash = proposer_14.hash();
    let proposer_15 = proposer_block!(vec![proposer_8.hash()], vec![transaction_10.hash()]);
    handle_block!(proposer_15);
    parent_hash = proposer_15.hash();
    assert_eq!(blockchain.unreferred_proposers().len(),1);
    assert!(blockchain.unreferred_proposers().contains(&proposer_15.hash()));
    assert_eq!(blockchain.unreferred_transactions().len(),0);
    for chain_number in 0..config::NUM_VOTER_CHAINS {
        let v = voter_block!(chain_number, blockchain.best_voter(chain_number as usize), vec![
        proposer_10.hash(),proposer_11.hash(),proposer_12.hash(),proposer_13.hash(),proposer_14.hash(),proposer_15.hash()]);
        handle_block!(v);
        //the fork becomes the best
        assert_eq!(v.hash(), blockchain.best_voter(chain_number as usize));
    }

    check_transaction_output!(transaction_1, true);
    check_transaction_output!(transaction_2, false);//tx 4 spends outputs of tx 2
    check_transaction_output!(transaction_3, true);
    //check_transaction_output!(transaction_4, true);//we don't check transaction 4 since we may spend it in transaction 8
    check_transaction_output!(transaction_5, true);
    check_transaction_output!(transaction_6, true);
    check_transaction_output!(transaction_7, true);
    check_transaction_output!(transaction_8, true);
    check_transaction_output!(transaction_9, true);
    check_transaction_output!(transaction_10, true);
    check_transaction_output!(transaction_invalid, false);
    assert_eq!(wallet.balance().unwrap(), 1+ico_number);

    let ledger = blockchain.proposer_transaction_in_ledger(100).unwrap();
    let ledger_proposer: Vec<H256> = ledger.into_iter().map(|x|x.0).collect();
    assert_eq!(ledger_proposer, vec![
        *config::PROPOSER_GENESIS_HASH,
        proposer_1.hash(),
        proposer_1_fork.hash(), proposer_1_fork_2.hash(),
        proposer_2.hash(),
        proposer_9.hash(),//since this is leader in the first place
        proposer_10.hash(),//since this is leader in the first place
        proposer_11.hash(),//since this is leader in the first place
        proposer_12.hash(),//since this is leader in the first place
        proposer_2_fork.hash(),//since 13 refer 4 first
        proposer_3.hash(),
        proposer_3_fork.hash(),
        proposer_4.hash(),//since 13 refer 6 second
        proposer_4_fork.hash(), proposer_4_fork_2.hash(),
        proposer_5.hash(),
        proposer_6.hash(),
        proposer_13.hash(),
        proposer_14.hash(),
        proposer_7.hash(),
        proposer_8.hash(),
        proposer_15.hash(),
    ]);

    //insert previous proposer blocks multiple times to check robustness
    handle_block!(proposer_4);
    handle_block!(proposer_3);
    handle_block!(proposer_4);
    handle_block!(proposer_5);
    handle_block!(proposer_7);
    handle_block!(proposer_8);
    handle_block!(proposer_2);
    handle_block!(proposer_3);
    //half+2 voter chains vote for one fork, others remain unchanged
    let half_vote = config::NUM_VOTER_CHAINS/2+2;
    for chain_number in 0..half_vote {
        let v = voter_block!(chain_number, voter_parent_to_fork_2[chain_number as usize], vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![]);
        handle_block!(v);
        let v = voter_block!(chain_number, v.hash(), vec![proposer_3.hash(),proposer_4.hash(),proposer_5.hash(),proposer_6.hash(),proposer_7.hash(),proposer_8.hash()]);
        handle_block!(v);
        //the fork becomes the best
        assert_eq!(v.hash(), blockchain.best_voter(chain_number as usize));
    }
    let ledger = blockchain.proposer_transaction_in_ledger(100).unwrap();
    let ledger_proposer: Vec<H256> = ledger.into_iter().map(|x|x.0).collect();
    assert_eq!(ledger_proposer, vec![
        *config::PROPOSER_GENESIS_HASH,
        proposer_1.hash(),
        proposer_1_fork.hash(), proposer_1_fork_2.hash(),
        proposer_2.hash(),
        proposer_2_fork.hash(),
        proposer_3.hash(),
        proposer_4.hash(),
        proposer_5.hash(),
        proposer_3_fork.hash(),
        proposer_4_fork.hash(), proposer_4_fork_2.hash(),
        proposer_6.hash(),
        proposer_7.hash(),
        proposer_8.hash(),
    ]);
}
