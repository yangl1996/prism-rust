use prism::validation::*;
use prism::block::tests::{proposer_block, voter_block, transaction_block};
use prism::crypto::hash::tests::generate_random_hash;
use prism::blockdb::BlockDatabase;
use prism::blockchain::BlockChain;
use prism::config;
use prism::crypto::hash::Hashable;

macro_rules! assert_result {
        ( $left:expr, $right:pat ) => {{
            if let $right = $left {} else {
                panic!("Wrong validation result: left is {}", $left);
            }
        }};
    }

#[test]
fn validate_block() {
    let blockdb = BlockDatabase::new("/tmp/prism_test_validation_blockdb.rocksdb").unwrap();

    let blockchain = BlockChain::new("/tmp/prism_test_validation_blockchain.rocksdb").unwrap();

    let mut parent = blockchain.best_proposer();
    let mut timestamp = 1u128;

    let proposer_1 = proposer_block(parent, timestamp, vec![], vec![]);
    assert_result!(check_block(&proposer_1, &blockchain, &blockdb), BlockResult::Pass);
    blockdb.insert(&proposer_1).unwrap();
    blockchain.insert_block(&proposer_1).unwrap();
    assert_result!(check_block(&proposer_1, &blockchain, &blockdb), BlockResult::Duplicate);
    let proposer_ = proposer_block(generate_random_hash(), timestamp, vec![], vec![]);
    assert_result!(check_block(&proposer_, &blockchain, &blockdb),  BlockResult::MissingParent(_));
    let proposer_ = proposer_block(parent, timestamp, vec![generate_random_hash()], vec![]);
    assert_result!(check_block(&proposer_, &blockchain, &blockdb),  BlockResult::MissingReferences(_));
    let proposer_ = proposer_block(parent, timestamp, vec![], vec![generate_random_hash()]);
    assert_result!(check_block(&proposer_, &blockchain, &blockdb),  BlockResult::MissingReferences(_));

    parent = blockchain.best_proposer();
    timestamp += 1;

    let proposer_2 = proposer_block(parent, timestamp, vec![], vec![]);
    assert_result!(check_block(&proposer_2, &blockchain, &blockdb), BlockResult::Pass);
    blockdb.insert(&proposer_2).unwrap();
    blockchain.insert_block(&proposer_2).unwrap();

    parent = blockchain.best_proposer();
    timestamp += 1;

    for chain in 0..config::NUM_VOTER_CHAINS {
        let voter = voter_block(parent, timestamp, chain, blockchain.best_voter(chain as usize), vec![]);
        assert_result!(check_block(&voter, &blockchain, &blockdb), BlockResult::WrongVoteLevel);
        let voter = voter_block(parent, timestamp, chain, blockchain.best_voter(chain as usize), vec![proposer_1.hash()]);
        assert_result!(check_block(&voter, &blockchain, &blockdb), BlockResult::WrongVoteLevel);
        let voter = voter_block(parent, timestamp, chain, blockchain.best_voter(chain as usize), vec![proposer_1.hash(), proposer_2.hash()]);
        assert_result!(check_block(&voter, &blockchain, &blockdb), BlockResult::Pass);
        let voter = voter_block(parent, timestamp, chain, blockchain.best_voter(chain as usize), vec![proposer_2.hash()]);
        assert_result!(check_block(&voter, &blockchain, &blockdb), BlockResult::WrongVoteLevel);
        // vote's order matters
        let voter = voter_block(parent, timestamp, chain, blockchain.best_voter(chain as usize), vec![proposer_2.hash(), proposer_1.hash()]);
        assert_result!(check_block(&voter, &blockchain, &blockdb), BlockResult::WrongVoteLevel);
    }
}