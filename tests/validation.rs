use prism::block::tests::{proposer_block, transaction_block, voter_block};
use prism::blockchain::BlockChain;
use prism::blockdb::BlockDatabase;
use prism::config;
use prism::crypto::hash::tests::generate_random_hash;
use prism::crypto::hash::Hashable;
use prism::transaction::tests::{generate_random_coinid, generate_random_transaction};
use prism::transaction::{Input, Output, Transaction};
use prism::validation::{check_content_semantic, check_data_availability, check_coin_ownership, 
    check_header_signature, check_content_hash, BlockResult};
use std::cell::RefCell;

macro_rules! assert_result {
    ( $left:expr, $right:pat ) => {{
        if let $right = $left {
        } else {
            panic!("Wrong validation result: left is {}", $left);
        }
    }};
}

#[test]
fn validate_block() {
    let blockdb = BlockDatabase::new("/tmp/prism_test_validation_blockdb.rocksdb").unwrap();

    let blockchain = BlockChain::new("/tmp/prism_test_validation_blockchain.rocksdb").unwrap();

    let mut parent = blockchain.best_proposer().unwrap().0;
    let mut timestamp = 1u128;

    let proposer_1 = proposer_block(parent, timestamp, vec![], vec![]);

    assert_result!(
        check_coin_ownership(&proposer_1),
        BlockResult::Pass
    );

    assert_result!(
        check_header_signature(&proposer_1),
        BlockResult::Pass
    );

    assert_result!(
        check_content_hash(&proposer_1),
        BlockResult::Pass
    );

   
    // assert_result!(
    //     check_data_availability(&proposer_1, &blockchain, &blockdb),
    //     BlockResult::Pass
    // );
    // assert_result!(
    //     check_content_semantic(&proposer_1, &blockchain, &blockdb),
    //     BlockResult::Pass
    // );
    // blockdb.insert(&proposer_1).unwrap();
    // blockchain.insert_block(&proposer_1).unwrap();
    // // TODO: we remove the duplicate?
    // // assert_result!(check_data_availability(&proposer_1, &blockchain, &blockdb), BlockResult::Duplicate);
    // let proposer_ = proposer_block(generate_random_hash(), timestamp, vec![], vec![]);
    // assert_result!(
    //     check_data_availability(&proposer_, &blockchain, &blockdb),
    //     BlockResult::MissingReferences(_)
    // );
    // let proposer_ = proposer_block(parent, timestamp, vec![generate_random_hash()], vec![]);
    // assert_result!(
    //     check_data_availability(&proposer_, &blockchain, &blockdb),
    //     BlockResult::MissingReferences(_)
    // );
    // let proposer_ = proposer_block(parent, timestamp, vec![], vec![generate_random_hash()]);
    // assert_result!(
    //     check_data_availability(&proposer_, &blockchain, &blockdb),
    //     BlockResult::MissingReferences(_)
    // );

    // parent = blockchain.best_proposer().unwrap().0;
    // timestamp += 1;

    // let proposer_2 = proposer_block(parent, timestamp, vec![], vec![]);
    // assert_result!(
    //     check_data_availability(&proposer_2, &blockchain, &blockdb),
    //     BlockResult::Pass
    // );
    // assert_result!(
    //     check_content_semantic(&proposer_2, &blockchain, &blockdb),
    //     BlockResult::Pass
    // );
    // blockdb.insert(&proposer_2).unwrap();
    // blockchain.insert_block(&proposer_2).unwrap();

    // let proposer_2_fork = proposer_block(parent, timestamp, vec![proposer_2.hash()], vec![]);
    // assert_result!(
    //     check_content_semantic(&proposer_2_fork, &blockchain, &blockdb),
    //     BlockResult::WrongProposerRef
    // );

    // parent = blockchain.best_proposer().unwrap().0;
    // timestamp += 1;

    // for chain in 0..config::NUM_VOTER_CHAINS {
    //     let voter = voter_block(
    //         blockchain.best_voter(chain as usize).0,
    //         timestamp,
    //         chain,
    //         vec![],
    //     );
    //     assert_result!(
    //         check_content_semantic(&voter, &blockchain, &blockdb),
    //         BlockResult::WrongVoteLevel
    //     );
    //     let voter = voter_block(
    //         blockchain.best_voter(chain as usize).0,
    //         timestamp,
    //         chain,
    //         vec![proposer_1.hash()],
    //     );
    //     assert_result!(
    //         check_content_semantic(&voter, &blockchain, &blockdb),
    //         BlockResult::WrongVoteLevel
    //     );
    //     let voter = voter_block(
    //         blockchain.best_voter(chain as usize).0,
    //         timestamp,
    //         chain,
    //         vec![proposer_1.hash(), proposer_2.hash()],
    //     );
    //     assert_result!(
    //         check_content_semantic(&voter, &blockchain, &blockdb),
    //         BlockResult::Pass
    //     );
    //     let voter = voter_block(
    //         blockchain.best_voter(chain as usize).0,
    //         timestamp,
    //         chain,
    //         vec![proposer_2.hash()],
    //     );
    //     assert_result!(
    //         check_content_semantic(&voter, &blockchain, &blockdb),
    //         BlockResult::WrongVoteLevel
    //     );
    //     // vote's order matters
    //     let voter = voter_block(
    //         blockchain.best_voter(chain as usize).0,
    //         timestamp,
    //         chain,
    //         vec![proposer_2.hash(), proposer_1.hash()],
    //     );
    //     assert_result!(
    //         check_content_semantic(&voter, &blockchain, &blockdb),
    //         BlockResult::WrongVoteLevel
    //     );
    //     if chain > 0 {
    //         let voter = voter_block(
    //             blockchain.best_voter(chain as usize).0,
    //             timestamp,
    //             chain - 1,
    //             vec![proposer_2.hash(), proposer_1.hash()],
    //         );
    //         assert_result!(
    //             check_content_semantic(&voter, &blockchain, &blockdb),
    //             BlockResult::WrongChainNumber
    //         );
    //     }
    // }

    // timestamp += 1;

    // let invalid_tx = Transaction {
    //     input: vec![],
    //     ..generate_random_transaction()
    // };
    // let transaction_1 = transaction_block(parent, timestamp, vec![invalid_tx]);
    // assert_result!(
    //     check_content_semantic(&transaction_1, &blockchain, &blockdb),
    //     BlockResult::EmptyTransaction
    // );
    // let invalid_tx = Transaction {
    //     output: vec![],
    //     ..generate_random_transaction()
    // };
    // let transaction_2 = transaction_block(parent, timestamp, vec![invalid_tx]);
    // assert_result!(
    //     check_content_semantic(&transaction_2, &blockchain, &blockdb),
    //     BlockResult::EmptyTransaction
    // );
    // let invalid_tx = Transaction {
    //     input: vec![Input {
    //         coin: generate_random_coinid(),
    //         value: 0,
    //         owner: [9u8; 32].into(),
    //     }],
    //     ..generate_random_transaction()
    // };
    // let transaction_3 = transaction_block(parent, timestamp, vec![invalid_tx]);
    // assert_result!(
    //     check_content_semantic(&transaction_3, &blockchain, &blockdb),
    //     BlockResult::ZeroValue
    // );
    // let invalid_tx = Transaction {
    //     output: vec![Output {
    //         value: 0,
    //         recipient: [9u8; 32].into(),
    //     }],
    //     ..generate_random_transaction()
    // };
    // let transaction_4 = transaction_block(parent, timestamp, vec![invalid_tx]);
    // assert_result!(
    //     check_content_semantic(&transaction_4, &blockchain, &blockdb),
    //     BlockResult::ZeroValue
    // );
    // let invalid_tx = Transaction {
    //     input: vec![Input {
    //         coin: generate_random_coinid(),
    //         value: 1,
    //         owner: [9u8; 32].into(),
    //     }],
    //     output: vec![Output {
    //         value: 2,
    //         recipient: [9u8; 32].into(),
    //     }],
    //     authorization: vec![],
    //     hash: RefCell::new(None),
    // };
    // let transaction_5 = transaction_block(parent, timestamp, vec![invalid_tx]);
    // assert_result!(
    //     check_content_semantic(&transaction_5, &blockchain, &blockdb),
    //     BlockResult::InsufficientInput
    // );
    // let invalid_tx = Transaction {
    //     input: vec![Input {
    //         coin: generate_random_coinid(),
    //         value: 2,
    //         owner: [9u8; 32].into(),
    //     }],
    //     output: vec![Output {
    //         value: 2,
    //         recipient: [9u8; 32].into(),
    //     }],
    //     authorization: vec![],
    //     hash: RefCell::new(None),
    // };
    // let transaction_6 = transaction_block(parent, timestamp, vec![invalid_tx]);
    // assert_result!(
    //     check_content_semantic(&transaction_6, &blockchain, &blockdb),
    //     BlockResult::WrongSignature
    // );
}
