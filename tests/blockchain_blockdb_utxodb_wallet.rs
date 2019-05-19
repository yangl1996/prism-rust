use prism::blockchain::BlockChain;
use prism::wallet::Wallet;
use prism::blockdb::BlockDatabase;
use prism::utxodb::UtxoDatabase;
use prism::block::{Block, Content, proposer};
use prism::crypto::hash::Hashable;
use prism::transaction::Transaction;

#[test]
fn integration() {
    let blockdb = BlockDatabase::new("/tmp/prism_test_integration_blockdb.rocksdb").unwrap();

    let blockchain = BlockChain::new("/tmp/prism_test_integration_blockchain.rocksdb").unwrap();

    let utxodb = UtxoDatabase::new("/tmp/prism_test_integration_utxodb.rocksdb").unwrap();

    let wallet = Wallet::new("/tmp/prism_test_integration_walletdb.rocksdb").unwrap();

    for timestamp in 0u64..1 {
        let parent = blockchain.best_proposer();
        let block = Block::new(parent, timestamp,0,[0u8;32].into(), vec![],
                   Content::Proposer(proposer::Content {
                       transaction_refs: vec![],
                       proposer_refs: vec![],
                   }), [0u8;32], [255u8;32].into());
        blockdb.insert(&block).expect(&format!("fail at inserting block {} into blockdb",timestamp));
        let diff = blockchain.insert_block(&block).expect(&format!("fail at inserting block {} into blockchain",timestamp));
        assert_eq!(blockchain.best_proposer(), block.hash());
        assert!(blockchain.unreferred_proposer().contains(&block.hash()));

        // I copy part of handler here.
        // If I just want to use part of handler, how can I do it?
        let mut add: Vec<Transaction> = vec![];
        let mut remove: Vec<Transaction> = vec![];
        for hash in diff.0 {
            let block = blockdb.get(&hash).unwrap().unwrap();
            let content = match block.content {
                Content::Transaction(data) => data,
                _ => unreachable!(),
            };
            let mut transactions = content.transactions.clone();
            add.append(&mut transactions);
        }
        for hash in diff.1 {
            let block = blockdb.get(&hash).unwrap().unwrap();
            let content = match block.content {
                Content::Transaction(data) => data,
                _ => unreachable!(),
            };
            let mut transactions = content.transactions.clone();
            remove.append(&mut transactions);
        }

        let coin_diff = utxodb.apply_diff(&add, &remove).expect(&format!("fail at updating utxo of block {}",timestamp));
        wallet.update(&coin_diff.0, &coin_diff.1).expect(&format!("fail at updating wallet of block {}",timestamp));
    }
}