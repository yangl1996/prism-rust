use prism::blockdb::BlockDatabase;
use prism::blockchain::BlockChain;
use prism::miner::memory_pool::MemoryPool;
use prism::miner::miner::ContextUpdateSignal;
use prism::miner::miner;
use prism::block::{Block, Content};
use prism::crypto::hash::{Hashable, H256};
use prism::config::NUM_VOTER_CHAINS;

use std::sync::{Mutex, mpsc, Arc};
use std::time::{Duration, Instant};
use std::{thread, time};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

const REPEAT: usize = 200;
const PROP_BLOCK_SIZE: u32 = 50;
const DEFAULT_BLOCKDB: &str = "/tmp/test_mining.rocksdb";

fn main() {

    // init database
    let blockdb_path = std::path::Path::new(&DEFAULT_BLOCKDB);
    let blockdb = BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = Arc::new(blockdb);

    // init blockchain
    let blockchain = BlockChain::new(NUM_VOTER_CHAINS);
    let blockchain = Arc::new(Mutex::new(blockchain));

    // init mempool
    let mempool = MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    // init the channels for the miner
    let (ctx_update_sink, ctx_update_source) = mpsc::channel();
    let (mined_block_sink, mined_block_source) = mpsc::channel();

    let (miner_ctx, miner_h) = miner::new(&mempool, &blockchain, &blockdb, mined_block_sink, ctx_update_source);
    miner_ctx.start();


    // blockchain parents
    let blockchain_read = blockchain.lock().unwrap();
    let mut proposer_best_block: H256 = blockchain_read.proposer_tree.best_block;
    let mut voter_best_blocks: Vec<H256> = (0..NUM_VOTER_CHAINS).map( |i| blockchain_read.voter_chains[i as usize].best_block).collect();// Currently the voter genesis blocks.
    drop(blockchain_read);

    // blocktype count
    let mut tx_block_no: u32 = 0;
    let mut prop_block_no: u32 = 0;
    let mut voter_block_no: u32 = 0;


    // step time
    let mut bc_time_ns: f64 = 0.0;
    let mut db_time_ns: f64 = 0.0;
    let mut mu_time_ns: f64 = 0.0; //miner content update time

    let mut mm_time_ns: f64 = 0.0; //miner mining update time
    let start = Instant::now();


    // Mine a block
    let mut mm_end = Instant::now();
    let mut mm_start = Instant::now();
    miner_h.step();
    loop {
        match mined_block_source.try_recv() {
            Ok(block) => {
                let mut mm_end = Instant::now();
                mm_time_ns += mm_end.duration_since(mm_start).as_micros() as f64;


                //1. Inserting the block in the blockchain
                let mut blockchain_write = blockchain.lock().unwrap();
                let bc_start = Instant::now();
                blockchain_write.insert_node(&block);
                let bc_end = Instant::now();
                bc_time_ns += bc_end.duration_since(bc_start).as_micros() as f64;

                //2. Inserting in db
                let db_start = Instant::now();
                let block_hash = block.hash();
                blockdb.insert(&block_hash, &block);
                let db_end = Instant::now();
                db_time_ns += db_end.duration_since(db_start).as_micros() as f64;

                //3. Change the miner content

                let mu_start = Instant::now();
                ctx_update_sink.send(ContextUpdateSignal::NewContent);
                let mu_end = Instant::now();
                mu_time_ns += mu_end.duration_since(mu_start).as_micros() as f64;


                //4. Testing
                assert_eq!(block.header.parent_hash, proposer_best_block);

                match block.content {
                    Content::Transaction(_) => {
                        tx_block_no +=1;
                    },
                    Content::Proposer(_) => {
                        prop_block_no +=1;
                        proposer_best_block = block_hash;

                        let end = Instant::now();
                        let time = end.duration_since(start).as_micros() as f64;
                        println!("For level {}, total {}, bc update {}, db update {}, miner update {}, mining  time {}", prop_block_no,
                                 (time / 1000000.0) / (prop_block_no as f64),
                                 (bc_time_ns / 1000000.0) / (prop_block_no as f64),
                                 (db_time_ns / 1000000.0) / (prop_block_no as f64),
                                 (mu_time_ns / 1000000.0) / (prop_block_no as f64),
                                 (mm_time_ns / 1000000.0) / (prop_block_no as f64), );

                    },
                    Content::Voter(c) => {
                        voter_block_no+=1;
                        let chain_number = c.chain_number as usize;
//                        assert_eq!(c.voter_parent_hash, voter_best_blocks[chain_number]);
                        voter_best_blocks[chain_number] = block_hash;
                    },
                }
                drop(blockchain_write);

                let mut mm_start = Instant::now();
                miner_h.step(); //step only when the previous block is mined
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Miner context update channel detached"),

        }
    }




//    let two_seconds = Duration::from_secs(2);
//
//    // Parking the main thread so that the miner thread can run
//    loop {
////        thread::sleep(two_seconds);
////        println!("Time spent on main thread : {:?}", Instant::now()- beginning_park);
//    }
}
