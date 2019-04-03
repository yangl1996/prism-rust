use crate::block::Block;
use crate::blockdb::BlockDatabase;
use crate::blockchain::BlockChain;
use crate::crypto::hash::Hashable;
use std::sync::Mutex;

pub fn new_block(block: Block, db: &BlockDatabase, chain: &Mutex<BlockChain>) {
    // TODO: for now, we assume that blocks appear in order
    let hash = block.hash();

    // insert the new block into the blockchain
    let mut chain = chain.lock().unwrap();
    chain.insert_node(&block);
    drop(chain);

    // insert the new block into the blockdb
    db.insert(&hash, &block);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_one_block() {
        // initialize 
        let blockchain = BlockChain::new();
        let database = BlockDatabase::new(std::path::Path::new("/tmp/prism_rule_new_block_tests_add_block"));
        let database = Mutex::new(database);

    }
}
