use std::sync::{mpsc, Mutex, Arc};
use super::message;
use super::peer;
use std::thread;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;

pub struct Context {
    msg_chan: Arc<Mutex<mpsc::Receiver<(message::Message, peer::Handle)>>>,
    num_worker: usize,
    chain: Arc<Mutex<BlockChain>>,
    blockdb: Arc<BlockDatabase>,
}

pub fn new(num_worker: usize, msg_src: mpsc::Receiver<(message::Message, peer::Handle)>,
           blockchain: Arc<Mutex<BlockChain>>, blockdb: Arc<BlockDatabase>) -> Context {
    let ctx = Context {
        msg_chan: Arc::new(Mutex::new(msg_src)),
        num_worker: num_worker,
        chain: Arc::clone(&blockchain),
        blockdb: Arc::clone(&blockdb),
    };
    return ctx;
}

fn worker_loop(msg_chan: Arc<Mutex<mpsc::Receiver<(message::Message, peer::Handle)>>>) {
    loop {
        let msg = msg_chan.lock().unwrap().recv().unwrap();
        // TODO: drop the lock here, or we are essentially blocking
        message::handle(&msg.1, &msg.0);
    }
}

impl Context {
    pub fn start(self) {
        for _ in 0..self.num_worker {
            let cloned_chan = Arc::clone(&self.msg_chan);
            thread::spawn(move || {
                worker_loop(cloned_chan);
            });
        }
    }
}
