use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use crate::miner::memory_pool::MemoryPool;
use crate::network::message;
use crate::network::server::Handle as ServerHandle;
use std::sync::Mutex;

pub fn new_validated_block(
    block: Block,
    mempool: &Mutex<MemoryPool>,
    db: &BlockDatabase,
    chain: &Mutex<BlockChain>,
    server: &ServerHandle,
) {
//    println!("New Block handled: {:?}.", block.hash());
//    match &block.content {
//        Content::Transaction(_) => {
//            println!("\tTx Block");
//        }
//        Content::Proposer(_) => {
//            println!("\tProp Block");
//        }
//        Content::Voter(_) => {
//            println!("\tVoter Block");
//        }
//        _ => (),
//    };
    // insert the new block into the blockdb
    db.insert(&block).unwrap();

    // if this block is a tx_block, remove transactions from mempool
    match &block.content {
        Content::Transaction(content) => {
            let mut mempool = mempool.lock().unwrap();
            for tx in content.transactions.iter() {
                mempool.remove_by_hash(&tx.hash());
                for input in tx.input.iter() {
                    mempool.remove_by_input(input);
                }
            }
        }
        _ => (),
    };

    // insert the new block into the blockchain
    let mut chain = chain.lock().unwrap();
    chain.insert_node(&block);
    drop(chain);

    // tell the neighbors that we have a new block
    server.broadcast(message::Message::NewBlockHashes(vec![block.hash()]));
}
