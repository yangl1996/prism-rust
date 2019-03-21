use prism::validation::single_transaction::{ValidatorCollection, NonEmptyValidator, TransactionValidator, InputInStateValidator};
use prism::miner::memory_pool::MemoryPool;
use prism::state::StateStorage;
use prism::transaction::{generator as transaction_generator, Transaction, IndexedTransaction};
use prism::crypto::hash::Hashable;
use std::sync::{Arc, RwLock};

/// tests the integration of validator, mempool, and state. not a functional handler
#[test]
fn combine_validator_mempool_state() {

    let mut mempool: Arc<RwLock<MemoryPool>> = Arc::new(RwLock::new(MemoryPool::new()));
    let mut state: Arc<RwLock<StateStorage>> = Arc::new(RwLock::new(StateStorage::new()));
    // A toy validator: only have two simple validators, a non-empty validator, and a input-in-state validator
    // Don't know if we need Arc and RwLock for validator? How? Lock for individual or whole validator?
    let validator = ValidatorCollection::new(
        vec![Box::new(NonEmptyValidator{}), Box::new(InputInStateValidator::new(state.clone()))]
    );
    // we generate 50 toy transactions, and validate, and insert to mempool
    for i in 0..50 {
        // unconfirmed transaction received
        let tx: Transaction = transaction_generator::random_transaction_builder().into();
        // since now state is empty, we have to add fake states, which is tx.input, you may comment this block and see that validator rejects these transactions
        {
            let mut state = state.write().unwrap();
            tx.input.iter().for_each(|input|state.insert(&input.hash, &input.index));
        }
        // validate the tx
        if validator.is_valid(&tx) {
            let mut mempool = mempool.write().unwrap();
            if !mempool.is_double_spend(&tx) {
                mempool.insert_verified(tx.into());
            } else {
                println!("reject a double spend transaction in mempool.")
            }
        } else {
            println!("transaction validation fails.")
        }
    }
    // assume the miner takes 50 transactions for mining
    let tx_to_mine = {
        let mempool = mempool.read().unwrap();
        mempool.get_n_transactions(50)
    };
    assert_eq!(tx_to_mine.len(), 50);
    // just assume these transactions are mined in a block (miner and blockchain is not involved in this example)
    // so we remove them from mempool
    {
        let mut mempool = mempool.write().unwrap();
        tx_to_mine.iter().for_each(|tx| {
            for input in &tx.input {
                mempool.remove_by_prevout(input);
            };
        });
    }
    // we remove the spent coins and we add transactions outputs to state
    {
        let mut state = state.write().unwrap();
        // we remove the spent coins
        tx_to_mine.iter().for_each(|tx| tx.input.iter().for_each(|input|state.remove(&input.hash, &input.index)));
        // now state should be empty
        assert!(state.is_empty());
        // we add transactions outputs to state
        tx_to_mine.iter().for_each(|tx| state.add(tx));
    }

    //check these transactions are in the state
    {
        let state = state.read().unwrap();
        assert!(tx_to_mine.iter().all(|tx| {
            let hash = tx.hash();
            (0..tx.output.len()).all(|i| state.contains(&hash, &(i as u32)))
        }));
    }
    //check these transactions are not in mempool
    {
        let mempool = mempool.read().unwrap();
        assert!(tx_to_mine.iter().all(|tx| {
            let hash = tx.hash();
            !mempool.contains(&hash)
        }));
    }
}