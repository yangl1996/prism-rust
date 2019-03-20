use prism::validation::single_transaction::{ValidatorCollection, NonEmptyValidator, TransactionValidator};
use prism::miner::memory_pool::MemoryPool;
use prism::state::StateStorage;
use prism::transaction::{generator as transaction_generator, Transaction, IndexedTransaction};
use prism::crypto::hash::Hashable;

#[test]
fn combine_validator_mempool_state() {
    /// A toy validator: only have one simple validator, non-empty validator
    let validator = ValidatorCollection::new(vec![Box::new(NonEmptyValidator{})]);
    let mut mempool = MemoryPool::new();
    let mut state = StateStorage::new();
    // we generate 100 toy transactions
    for i in 0..100 {
        // unconfirmed transaction received
        let tx: Transaction = transaction_generator::random_transaction_builder().into();
        // validate the tx
        if validator.is_valid(&tx) {
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
    let tx_to_mine = mempool.get_n_transactions(50);
    // just assume these transactions are mined and confirmed (miner and blockchain is not involved in this example)
    // so we remove them from mempool
    tx_to_mine.iter().for_each(|tx|{
        for input in &tx.input {
            mempool.remove_by_prevout(input);
        };
    });
    // then we add them to state
    tx_to_mine.iter().for_each(|tx|state.add(tx));
    //check these transactions are in the state
    assert!(tx_to_mine.iter().all(|tx|{
        let hash = tx.hash();
        (0..tx.output.len()).all(|i|state.contains(&hash, &(i as u32)))
    }));
    //check these transactions are not in mempool
    assert!(tx_to_mine.iter().all(|tx|{
        let hash = tx.hash();
        !mempool.contains(&hash)
    }));
}