pub mod memory_pool;

use crate::block::header::Header;
use crate::block::{proposer, transaction, voter};
use crate::block::{Block, Content};
use crate::block::pos_metadata::{Metadata, RandomSource, TimeStamp};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::config::*;
use crate::crypto::hash::{Hashable, H256};
use crate::crypto::merkle::MerkleTree;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::handler::new_validated_block;
use crate::network::message::Message;
use crate::network::server::Handle as ServerHandle;
use crate::validation::check_difficulty;
use crate::crypto::vrf::{VrfSecretKey, VrfPublicKey, VrfInput, vrf_evaluate, VrfOutput};
use crate::utxodb::Utxo;
use crate::wallet::Wallet;
use ed25519_dalek::Keypair;
use log::{trace, debug, info, warn};

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use memory_pool::MemoryPool;
use std::time;
use std::time::SystemTime;

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

enum ControlSignal {
    Start(u64), // the number controls the delta of interval between block generation
    Step,
    Exit,
}

#[derive(Ord, Eq, PartialOrd, PartialEq)]
pub enum ContextUpdateSignal {
    // TODO: New transaction comes, we update transaction block's content
    //NewTx,//should be called: mem pool change
    // New proposer block comes, we need to update all contents' parent
    NewProposerBlock,
    // New voter block comes, we need to update that voter chain
    NewVoterBlock(u16),
    // New transaction block comes, we need to update proposer content's tx ref
    NewTransactionBlock,
}

enum OperatingState {
    Paused,
    Run(u64),
    Step,
    ShutDown,
}

pub struct Context {
    blockdb: Arc<BlockDatabase>,
    blockchain: Arc<BlockChain>,
    wallet: Arc<Wallet>,
    mempool: Arc<Mutex<MemoryPool>>,
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    /// Channel for notifying miner of new content
    context_update_chan: Receiver<ContextUpdateSignal>,
    context_update_tx: Sender<ContextUpdateSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    /// Last timestamp this miner tried
    timestamp: TimeStamp,
    extra_content: [u8;32],
    /// Parent and level for proposer and voter chains
    parents: [(H256, u64); (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize],
    difficulties: [H256; (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize],
    random_sources: [RandomSource; (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize],
}

#[derive(Clone)]
pub struct Handle {
    // Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    mempool: &Arc<Mutex<MemoryPool>>,
    blockchain: &Arc<BlockChain>,
    blockdb: &Arc<BlockDatabase>,
    wallet: &Arc<Wallet>,
    ctx_update_source: Receiver<ContextUpdateSignal>,
    ctx_update_tx: &Sender<ContextUpdateSignal>,
    server: &ServerHandle,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let parents = [(H256::default(), 0); (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize];
    let difficulties = [H256::default(); (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize];
    let random_sources = [RandomSource::default(); (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize];

    let ctx = Context {
        blockdb: Arc::clone(blockdb),
        blockchain: Arc::clone(blockchain),
        wallet: Arc::clone(wallet),
        mempool: Arc::clone(mempool),
        control_chan: signal_chan_receiver,
        context_update_chan: ctx_update_source,
        context_update_tx: ctx_update_tx.clone(),
        operating_state: OperatingState::Paused,
        server: server.clone(),
        timestamp: 0,
        extra_content: [0; 32],
        parents,
        difficulties,
        random_sources,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    return (ctx, handle);
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, delta: u64) {
        self.control_chan
            .send(ControlSignal::Start(delta))
            .unwrap();
    }

    pub fn step(&self) {
        self.control_chan.send(ControlSignal::Step).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!(
                    "Miner starting in continuous mode with delta {}",
                    i
                );
                self.operating_state = OperatingState::Run(i);
            }
            ControlSignal::Step => {
                info!("Miner starting in stepping mode");
                self.operating_state = OperatingState::Step;
            }
        }
    }

    fn miner_loop(&mut self) {
        // tell ourself to update all context
        // equivalent to initialization of contexts
        self.context_update_tx
            .send(ContextUpdateSignal::NewProposerBlock).unwrap();
        for voter_chain in 0..NUM_VOTER_CHAINS {
            self.context_update_tx
                .send(ContextUpdateSignal::NewVoterBlock(voter_chain)).unwrap();
        }

        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // check and react to update signals
            let mut new_proposer_block: bool = false;
            let mut new_voter_block: BTreeSet<u16> = BTreeSet::new();
            for sig in self.context_update_chan.try_iter() {
                match sig {
                    ContextUpdateSignal::NewProposerBlock => new_proposer_block = true,
                    ContextUpdateSignal::NewVoterBlock(chain) => {
                        new_voter_block.insert(chain);
                    }
                    ContextUpdateSignal::NewTransactionBlock => {}
                };
            }

            // update the proposer parent and random source
            if new_proposer_block {
                let chain_id: usize = PROPOSER_INDEX as usize;
                self.parents[chain_id] = self.blockchain.best_proposer().unwrap();
                let block = self.blockdb.get(&self.parents[chain_id].0).unwrap();
                if let Some(block) = block {
                    self.difficulties[chain_id] = block.header.difficulty;
                    self.random_sources[chain_id] = block.header.pos_metadata.random_source;
                } else {
                    unreachable!();
                }
            }

            // update the voter parent and random source
            for voter_chain in new_voter_block {
                let chain_id: usize = (FIRST_VOTER_INDEX + voter_chain) as usize;
                self.parents[chain_id] = self.blockchain.best_voter(voter_chain as usize);
                let block = self.blockdb.get(&self.parents[chain_id].0).unwrap();
                if let Some(block) = block {
                    self.difficulties[chain_id] = block.header.difficulty;
                    self.random_sources[chain_id] = block.header.pos_metadata.random_source;
                } else {
                    unreachable!();
                }
            }

            // customizable mining time step
            // delta must be a multiple of DELTA
            // e.g. DELTA=0.1, we can set delta=0.1,0.2,0.3,...
            let delta = if let OperatingState::Run(delta) = self.operating_state {
                let delta = delta as TimeStamp;
                if delta!=0 && delta % DELTA == 0 {
                    delta
                } else {
                    DELTA
                }
            } else {
                DELTA
            };
            let timestamp = ( get_time() / delta + 1 ) * delta ;
            if timestamp > self.timestamp {
                self.timestamp = timestamp;
            } else {
                // the timestamp didn't proceed, we go over the loop again
                continue;
            }

            let mut mined_blocks = vec![];
            // Get all the coins that satisfy time−tau
            let time_minus_tau = if self.timestamp > TAU {
                self.timestamp - TAU
            } else {
                0
            };
            let coins = self.wallet.coins_before(time_minus_tau).unwrap();
            let coins_len = coins.len();
            
            let mining_manager: MiningManager = (&*self).into();
            for (utxo, keypair) in coins {
                let keypair = Keypair::from_bytes(&keypair).unwrap();
                trace!("Start mining at time {} with utxo {:?}", self.timestamp, utxo);
                mined_blocks.append(&mut mining_manager.start(utxo, keypair, 4));
            }
            // insert into our local database
            for mined_block in mined_blocks.iter() {
                PERFORMANCE_COUNTER.record_mine_block(mined_block);
                self.blockdb.insert(mined_block).unwrap();
                new_validated_block(
                    mined_block,
                    &self.mempool,
                    &self.blockdb,
                    &self.blockchain,
                    &self.server,
                    );
            }
            // sleep until the timestamp
            {
                let timestamp = self.timestamp;
                let (s, r) = unbounded();
                let mined_len = mined_blocks.len();
                thread::spawn(move || {
                    let start = get_time();
                    warn!("Plan to sleep {} ms,\tuse {} coins to mine,\tmined {} blocks.", timestamp as i128 - start as i128, coins_len, mined_len);
                    loop {
                        if get_time()>=timestamp {
                            break;
                        }
                        thread::sleep(time::Duration::from_millis(1));
                    }
                    s.send(()).unwrap();
                }
                );
                r.recv().unwrap();
            }
            // broadcast after sleeping
            for mined_block in mined_blocks.iter() {
                self.server
                    .broadcast(Message::NewBlockHashes(vec![mined_block.hash()]));
                // if we are stepping, pause the miner loop
                if let OperatingState::Step = self.operating_state {
                    self.operating_state = OperatingState::Paused;
                }
                // after we mined this block, we update the context based on this block
                match &mined_block.content {
                    Content::Proposer(_) => self
                        .context_update_tx
                        .send(ContextUpdateSignal::NewProposerBlock)
                        .unwrap(),
                    Content::Voter(content) => self
                        .context_update_tx
                        .send(ContextUpdateSignal::NewVoterBlock(content.chain_number))
                        .unwrap(),
                    Content::Transaction(_) => self
                        .context_update_tx
                        .send(ContextUpdateSignal::NewTransactionBlock)
                        .unwrap(),
                }
            }

        }
    }
}

#[derive(Clone)]
struct MiningManager {
    blockchain: Arc<BlockChain>,
    mempool: Arc<Mutex<MemoryPool>>,
    timestamp: TimeStamp,
    extra_content: Arc<[u8;32]>,
    /// Parent and level for proposer and voter chains
    parents: Arc<[(H256, u64); (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize]>,
    difficulties: Arc<[H256; (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize]>,
    random_sources: Arc<[RandomSource; (FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize]>,
    mined_chains: Arc<RwLock<BTreeSet<usize>>>,
}

impl std::convert::From<&Context> for MiningManager {
    fn from(other: &Context) -> MiningManager {
        MiningManager {
            blockchain: Arc::clone(&other.blockchain),
            mempool: Arc::clone(&other.mempool),
            timestamp: other.timestamp,
            extra_content: Arc::new(other.extra_content.clone()),
            parents: Arc::new(other.parents.clone()),
            difficulties: Arc::new(other.difficulties.clone()),
            random_sources: Arc::new(other.random_sources.clone()),
            mined_chains: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

impl MiningManager {
    fn start(&self, utxo: Utxo, keypair: Keypair, num_workers: usize) -> Vec<Block> {
        let utxo = Arc::new(utxo);
        let keypair = Arc::new(keypair);
        let (s, r) = unbounded();
        let mut threads = vec![];
        for i in 0..num_workers {
            let self_cloned = self.clone();
            let s_cloned = s.clone();
            let utxo_clone = Arc::clone(&utxo);
            let keypair_clone = Arc::clone(&keypair);
            threads.push(thread::spawn(move || {
                for chain_id in (i..(FIRST_VOTER_INDEX + NUM_VOTER_CHAINS) as usize).step_by(num_workers) {
                    //do the mining, get the result
                    if self_cloned.mined_chains.read().unwrap().contains(&chain_id) {
                        continue;
                    }
                    if let Some(block) = self_cloned.pos_mining(&utxo_clone, chain_id, &keypair_clone) {
                        s_cloned.send(block).unwrap();
                    }
                }
                drop(s_cloned);
            }));
        }
        /* maybe unnecessary since r.iter() will block
        for t in threads {
            t.join().unwrap();
        }
        */
        drop(s);
        let result: Vec<Block> = r.iter().collect();
        let mut mined_chains = self.mined_chains.write().unwrap();
        for block in result.iter() {
            match &block.content {
                Content::Proposer(_) => {
                    mined_chains.insert(PROPOSER_INDEX as usize);
                }
                Content::Voter(content) => {
                    mined_chains.insert((FIRST_VOTER_INDEX + content.chain_number) as usize);
                }
                _ => {}
            };
        }
        drop(mined_chains);
        result
    }

    fn pos_mining(&self, utxo: &Utxo, chain_id: usize, keypair: &Keypair) -> Option<Block> {
        let vrf_pubkey: VrfPublicKey = (&keypair.public).into();
        let input = VrfInput {
            random_source: self.random_sources[chain_id],
            time: self.timestamp,
            coin: utxo.coin.clone(),
        };
        let (vrf_value, vrf_proof) = vrf_evaluate(&vrf_pubkey, &keypair.secret, &input);
        // Check if we successfully mined a block
        if let Some(sortition_id) = check_difficulty(&vrf_value , &self.difficulties[chain_id], utxo.value) {//TODO difficulty * stake
            if chain_id as u16 != PROPOSER_INDEX && sortition_id != PROPOSER_INDEX {
                // on voter chain, the sortition result should be PROPOSER_INDEX to match the
                // proposer mining rate
                return None;
            }
            // Update random source every GAMMA
            let random_source = if (self.parents[chain_id].1 + 1) % GAMMA == 0 {
                (&vrf_value).into()
            } else {
                self.random_sources[chain_id]
            };
            // Create metadata
            let metadata = Metadata {
                vrf_proof,
                vrf_output: vrf_value,
                vrf_pubkey,
                utxo: utxo.clone(),
                parent_random_source: self.random_sources[chain_id],
                timestamp: self.timestamp,
                random_source,
            };
            // Create content
            let content = if chain_id as u16 == PROPOSER_INDEX {
                if sortition_id == PROPOSER_INDEX {
                    let transaction_refs = self.blockchain.unreferred_transactions();
                    let proposer_refs = self.blockchain.unreferred_proposers();
                    // TODO remove parent from refs
                    Content::Proposer(proposer::Content {
                        transaction_refs,
                        proposer_refs,
                    })
                } else {
                    // sortition result is a transaction block
                    let mempool = self.mempool.lock().unwrap();
                    let transactions = mempool.get_transactions(TX_BLOCK_TRANSACTIONS);
                    drop(mempool);
                    Content::Transaction(transaction::Content {
                        transactions,
                    })
                }
            } else {
                let votes = self.blockchain.unvoted_proposer(&self.parents[chain_id].0, &self.blockchain.best_proposer().unwrap().0, self.timestamp).unwrap();
                Content::Voter(voter::Content {
                    chain_number: chain_id as u16 - FIRST_VOTER_INDEX,
                    votes,
                })
            };
            let difficulty = self.difficulties[chain_id];// FUTURE: Adpative difficulty here
            // Create header
            let mut header = Header {
                parent: self.parents[chain_id].0,
                pos_metadata: metadata,
                content_root: content.hash(),
                extra_content: (*self.extra_content).clone(),
                difficulty,
                header_signature: vec![],
            };
            // Update signature of header
            let raw_unsigned = bincode::serialize(&header).unwrap();
            header.header_signature = keypair.sign(&raw_unsigned).to_bytes().to_vec();
            // Create a block
            let mined_block = Block::from_header(
                header,
                content,
                );
            Some(mined_block)
        } else {
            None
        }
    }
}


/// Get the current UNIX timestamp
fn get_time() -> TimeStamp {
    let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    match cur_time {
        Ok(v) => {
            return v.as_millis();
        }
        Err(e) => println!("Error parsing time: {:?}", e),
    }
    // TODO: there should be a better way of handling this, or just unwrap and panic
    return 0;
}

#[cfg(test)]
mod tests {
    use super::memory_pool::MemoryPool;
    use super::{Context, OperatingState};
    use crate::block::tests::{proposer_block, transaction_block, voter_block};
    use crate::block::{proposer, transaction, voter, Content};
    use crate::blockchain::BlockChain;
    use crate::blockdb::BlockDatabase;
    use crate::config;
    use crate::crypto::hash::{Hashable, H256};
    use crate::crypto::merkle::MerkleTree;
    use crate::network::server;
    use crate::transaction::tests as tx_generator;
    use crate::validation::{check_block, BlockResult};
    use std::sync::{Arc, Mutex};

    /*
    #[test]
    fn difficulty() {
        // Initialize a blockchain with 10 voter chains.
        let mempool = Arc::new(Mutex::new(MemoryPool::new()));
        let blockchain = Arc::new(Mutex::new(BlockChain::new(10)));
        let db = Arc::new(BlockDatabase::new(
            &std::path::Path::new("/tmp/prism_miner_check_get_difficulty.rocksdb")).unwrap());
        let (ctx_update_s, ctx_update_r) = channel();
        let (sender, receiver) = channel();
        let (ctx, handle) = new(&mempool, &blockchain, &db, sender, ctx_update_r);
        ctx.start();
        handle.step();
        let block1 = receiver.recv().unwrap();
        handle.exit();
        assert_eq!(block1.header.difficulty, DEFAULT_DIFFICULTY);
    }
    */

    /* this test is commented out for now, since it requires that we add the newly mined blocks to
       the db and the blockchain. if we add those, the test becomes an integration test, and no
       longer fits here.
       TODO: Gerui: but only here can we call a private function
    */

    /*
    // test assemble block and check the block passes validation
    #[test]
    fn assemble_block() {
        let blockdb = BlockDatabase::new("/tmp/prism_test_miner_blockdb.rocksdb").unwrap();
        let blockdb = Arc::new(blockdb);

        let blockchain = BlockChain::new("/tmp/prism_test_miner_blockchain.rocksdb").unwrap();
        let blockchain = Arc::new(blockchain);

        let mempool = Arc::new(Mutex::new(MemoryPool::new(100)));
        let (signal_chan_sender, signal_chan_receiver) = channel();
        let (ctx_update_sink, ctx_update_source) = channel();
        let (msg_tx, msg_rx) = channel();

        let parent = blockchain.best_proposer().unwrap().0;
        let mut content = vec![];
        content.push(Content::Proposer(proposer::Content::new( vec![], vec![] )));
        content.push(Content::Transaction(transaction::Content::new(vec![])));
        let voter_parent_hash: Vec<H256> = (0..config::NUM_VOTER_CHAINS)
            .map(|i| blockchain.best_voter(i as usize).0)
            .collect();
        let proposer_block_votes: Vec<Vec<H256>> = (0..config::NUM_VOTER_CHAINS)
            .map(|i| {
                blockchain
                    .unvoted_proposer(&voter_parent_hash[i as usize], &parent)
                    .unwrap()
                    .clone()
            })
            .collect();
        for (i, (voter_parent, proposer_block_votes)) in voter_parent_hash
            .into_iter()
            .zip(proposer_block_votes.into_iter())
            .enumerate()
            {
                content.push(Content::Voter(voter::Content::new(
                    i as u16,
                    voter_parent,
                    proposer_block_votes,
                )));
            }
        let (server_ctx, server) = server::new(
            std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 8080),
            msg_tx,
        ).unwrap();
        let mut miner = Context {
            mempool: Arc::clone(&mempool),
            blockchain: Arc::clone(&blockchain),
            blockdb: Arc::clone(&blockdb),
            control_chan: signal_chan_receiver,
            context_update_chan: ctx_update_source,
            proposer_parent_hash: parent,
            content,
            difficulty: *config::DEFAULT_DIFFICULTY,
            operating_state: OperatingState::Paused,
            server,
        };
        for nonce in 0..100 {
            let mut header = miner.create_header();
            header.nonce = nonce;
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
        for chain in 0..config::NUM_VOTER_CHAINS {
            let voter = voter_block(parent, 3,chain, blockchain.best_voter(chain as usize).0, vec![]);
            blockdb.insert(&voter);
            blockchain.insert_block(&voter);
            miner.update_voter_content(chain);
            let header = miner.create_header();
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
        miner.update_refed_transaction();
        for nonce in 0..100 {
            let mut header = miner.create_header();
            header.nonce = nonce;
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
        let proposer_1 = proposer_block(parent, 3, vec![], vec![]);
        blockdb.insert(&proposer_1);
        blockchain.insert_block(&proposer_1);
        miner.update_all_contents();
        for nonce in 0..100 {
            let mut header = miner.create_header();
            header.nonce = nonce;
            // Here, we assume difficulty is large enough s.t. we can get a block every time
            let block = miner.assemble_block(header);
            let result = check_block(&block, &blockchain, &blockdb);
            if let BlockResult::Pass = result {}
            else {
                panic!("Miner mine a block that doesn't pass validation!\n\tResult: {:?},\n\tBlock: {:?}\n\tContent Hash: {}", result, block, block.content.hash() );
            }
        }
    }
    */
}
