use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use std::sync::Mutex;
use crate::network::server::Handle as ServerHandle;
use crate::network::message;

pub fn new_validated_block(block: Block, db: &BlockDatabase, chain: &Mutex<BlockChain>, server: &ServerHandle) {
    // insert the new block into the blockdb
    db.insert(&block).unwrap();

    // insert the new block into the blockchain
    let mut chain = chain.lock().unwrap();
    chain.insert_node(&block);
    drop(chain);

    // tell the neighbors that we have a new block
    server.broadcast(message::Message::NewBlockHashes(vec![block.hash()]));
}
