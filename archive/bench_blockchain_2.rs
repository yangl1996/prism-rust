use prism::crypto::hash::Hashable;
use prism::blockchain::{BlockChain};
use prism::block::{Block};
use prism::blockchain::utils as bc_utils;
use std::time::Instant;
use std::{thread, time};

const REPEAT: usize = 200;
const PROP_BLOCK_SIZE: u32 = 500;
const NUM_VOTING_CHAIN: usize = 1000;

fn main() {

    let mut blockchain = BlockChain::new(NUM_VOTING_CHAIN as u16);

    // generate data
    println!("Generating blocks in the blockchain");
    let start = Instant::now();
    for j in 0..REPEAT {
        let proposer_parent_hash = blockchain.get_proposer_best_block();
        let tx_blocks: Vec<Block> = bc_utils::test_tx_blocks_with_parent_hash(PROP_BLOCK_SIZE, proposer_parent_hash);
        for tx_block in tx_blocks.iter() {
            blockchain.insert_node(&tx_block);
        }
        // generate proposer block with above transaction blocks
        let prop_block = bc_utils::test_prop_block(proposer_parent_hash, tx_blocks.iter().map( |x| x.hash()).collect(), vec![]);
        blockchain.insert_node(&prop_block);
        // votes on the proposer block
        for i in 0..NUM_VOTING_CHAIN {
            let voter_parent_hash = blockchain.get_voter_best_block(i as u16);
            let voter_block = bc_utils::test_voter_block(proposer_parent_hash, i as u16, voter_parent_hash, vec![prop_block.hash().clone()]);
            blockchain.insert_node(&voter_block);
        }
        if j % 5 == 4 {
            let end = Instant::now();
            let time = end.duration_since(start).as_micros() as f64;
            let throughput = j as f64 / (time / 1000000.0);
            println!("At level:{}. Insert  {:.2} levels/s", j, throughput);
        }
    }

    println!("Total nodes {}", blockchain.graph.node_count());
    println!("Total edges {}", blockchain.graph.edge_count());
    println!("Leader block len {}", blockchain.proposer_tree.leader_nodes.len());
    println!("All votes  len {}", blockchain.proposer_tree.all_votes.len());
    println!("prop_nodes  len {}", blockchain.proposer_tree.prop_nodes.len());
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
    drop(blockchain.tx_pool);
    println!("Dropped tx_pool structure");
    thread::sleep(time::Duration::from_secs(10));

    println!("Dropping proposer_node_data structure in 10");
    thread::sleep(time::Duration::from_secs(10));
    drop(blockchain.proposer_node_data);
    println!("Dropped proposer_node_data structure");
    thread::sleep(time::Duration::from_secs(10));

    println!("Dropping voter_node_data structure in 10");
    thread::sleep(time::Duration::from_secs(10));
    drop(blockchain.voter_node_data);
    println!("Dropped voter_node_data structure");
    thread::sleep(time::Duration::from_secs(10));
}
