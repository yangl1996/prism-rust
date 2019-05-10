use prism::crypto::hash::Hashable;
use prism::blockchain::{BlockChain};
use prism::blockchain::database::BlockChainDatabase;
use prism::block::{Block};
use prism::blockchain::utils as bc_utils;
use std::time::Instant;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

const LEVELS: usize = 200;
const PROP_BLOCK_SIZE: u32 = 100;
const NUM_VOTER_CHAINS: u16 = 0;

fn main() {


    let blockchain_db_path = std::path::Path::new("/tmp/blockchain_test1.rocksdb");
    let blockchain_db = BlockChainDatabase::new(blockchain_db_path).unwrap();
    let blockchain_db = Arc::new(Mutex::new(blockchain_db));

    // Initialize a blockchain with 10  voter chains.
    let (state_update_sink, _state_update_source) = mpsc::channel();
    let mut blockchain = BlockChain::new(blockchain_db, NUM_VOTER_CHAINS, state_update_sink);

    // generate data
    println!("Generating blocks in the blockchain");
    let start = Instant::now();
    for j in 0..LEVELS {
        let proposer_parent = blockchain.get_proposer_best_block();
        let tx_blocks: Vec<Block> = bc_utils::test_tx_blocks_with_parent(PROP_BLOCK_SIZE, proposer_parent);
        for tx_block in tx_blocks.iter() {
            blockchain.insert_node(&tx_block);
        }
        // generate proposer block with above transaction blocks
        let prop_block = bc_utils::test_prop_block(proposer_parent, tx_blocks.iter().map( |x| x.hash()).collect(), vec![]);
        blockchain.insert_node(&prop_block);
        // votes on the proposer block
        for i in 0..NUM_VOTER_CHAINS {
            let voter_parent = blockchain.get_voter_best_block(i as u16);
            let voter_block = bc_utils::test_voter_block(proposer_parent, i as u16, voter_parent, vec![prop_block.hash().clone()]);
            blockchain.insert_node(&voter_block);
        }
        if j % 5 == 4 {
            let end = Instant::now();
            let time = end.duration_since(start).as_micros() as f64;
            let throughput = j as f64 / (time / 1000000.0);
            println!("At level:{}. Insert  {:.2} levels/s", j, throughput);
//            println!("UnVotPropBlk size {}", blockchain.voter_chains[0].unvoted_proposer_blocks.len());
            println!("UnRevPropBlk size {}", blockchain.proposer_tree.unreferred.len());
            println!("UnRevTxBlk size {}", blockchain.tx_blocks.unreferred.len());
            println!("NotInled size {}", blockchain.tx_blocks.not_in_ledger.len());
        }
    }

    println!("Total edges {}", blockchain.graph.edge_count);
    println!("Leader block len {}", blockchain.proposer_tree.max_confirmed_level);
    println!("prop_nodes  len {}", blockchain.proposer_tree.best_level);
    println!("unrefferred  len {}", blockchain.proposer_tree.unreferred.len());

    println!("Dropping graph structure in 10");
    thread::sleep(time::Duration::from_secs(10));
    drop(blockchain.graph);
    println!("Dropped graph structure");
    thread::sleep(time::Duration::from_secs(10));

    println!("Dropping proposer_tree structure in 10");
    thread::sleep(time::Duration::from_secs(10));
    drop(blockchain.proposer_tree);
    println!("Dropped proposer_tree structure");
    thread::sleep(time::Duration::from_secs(10));

    println!("Dropping voter_chains structure in 10");
    thread::sleep(time::Duration::from_secs(10));
    drop(blockchain.voter_chains);
    println!("Dropped voter_chains structure");
    thread::sleep(time::Duration::from_secs(10));

    println!("Dropping tx_pool structure in 10");
    thread::sleep(time::Duration::from_secs(10));
    drop(blockchain.tx_blocks);
    println!("Dropped tx_pool structure");
    thread::sleep(time::Duration::from_secs(10));

    println!("Dropping node_data structure in 10");
    thread::sleep(time::Duration::from_secs(10));
    drop(blockchain.node_data);
    println!("Dropped node_data structure");
    thread::sleep(time::Duration::from_secs(10));
}
