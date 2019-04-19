use prism::block::Block;
use prism::blockchain::utils as bc_utils;
use prism::blockchain::BlockChain;
use prism::crypto::hash::Hashable;
use std::sync::mpsc;
use std::time::Instant;

const REPEAT: usize = 1000;
const PROP_BLOCK_SIZE: u32 = 500;
const NUM_VOTING_CHAIN: usize = 1000;

fn main() {
    let (state_update_sink, state_update_source) = mpsc::channel();
    let mut blockchain = BlockChain::new(NUM_VOTING_CHAIN as u16, state_update_sink);
    let mut all_tx_blocks: Vec<Vec<Block>> = vec![];

    let mut proposer_blocks: Vec<Block> = vec![];
    let mut proposer_parent_hash = blockchain.proposer_tree.best_block;

    let mut voter_blocks: Vec<Vec<Block>> = vec![];
    let mut voter_parent_hash = vec![];
    for i in 0..NUM_VOTING_CHAIN {
        voter_parent_hash.push(blockchain.voter_chains[i].best_block);
        voter_blocks.push(vec![]);
    }

    // generate data
    println!("Generating blocks in the blockchain");
    for _ in 0..REPEAT {
        // generate proposer block with random number of transaction blocks
        let tx_blocks: Vec<Block> =
            bc_utils::test_tx_blocks_with_parent_hash(PROP_BLOCK_SIZE, proposer_parent_hash);
        let prop_block = bc_utils::test_prop_block(
            proposer_parent_hash,
            tx_blocks.iter().map(|x| x.hash()).collect(),
            vec![],
        );
        // votes on the proposer block
        for i in 0..NUM_VOTING_CHAIN {
            let voter_block = bc_utils::test_voter_block(
                proposer_parent_hash,
                i as u16,
                voter_parent_hash[i],
                vec![prop_block.hash().clone()],
            );
            voter_parent_hash[i] = voter_block.hash();
            voter_blocks[i].push(voter_block);
        }
        // Updating parent blocks
        proposer_parent_hash = prop_block.hash();

        // Queuing up the blocks
        proposer_blocks.push(prop_block);
        all_tx_blocks.push(tx_blocks);
    }

    println!("Inserting blocks in the blockchain");
    println!("Required rate > 0.1 levels/s");
    let start = Instant::now();
    for i in 0..REPEAT {
        for tx_block in all_tx_blocks[i].iter() {
            blockchain.insert_node(&tx_block);
        }
        blockchain.insert_node(&proposer_blocks[i]);
        for j in 0..NUM_VOTING_CHAIN {
            blockchain.insert_node(&voter_blocks[j][i]);
        }
        if i % 50 == 49 {
            let end = Instant::now();
            let time = end.duration_since(start).as_micros() as f64;
            let throughput = i as f64 / (time / 1000000.0);
            println!("At level:{}. Insert  {:.2} levels/s", i, throughput);
        }
    }
}
