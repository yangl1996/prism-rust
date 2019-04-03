use std::sync::{mpsc, Mutex, Arc};
use log::{debug, error, info, trace, warn};
use super::message::{self, Message};
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
           blockchain: &Arc<Mutex<BlockChain>>, blockdb: &Arc<BlockDatabase>) -> Context {
    let ctx = Context {
        msg_chan: Arc::new(Mutex::new(msg_src)),
        num_worker: num_worker,
        chain: Arc::clone(blockchain),
        blockdb: Arc::clone(blockdb),
    };
    return ctx;
}

impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        let arc = Arc::new(self);
        for _ in 0..num_worker {
            let cloned = Arc::clone(&arc);
            thread::spawn(move || {
                cloned.worker_loop();
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let chan = self.msg_chan.lock().unwrap();
            let msg = chan.recv().unwrap();
            drop(chan);
            let (msg, peer) = msg;
            match msg {
                Message::Ping(nonce) => {
                    info!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                },
                Message::Pong(nonce) => {
                    info!("Pong: {}", nonce);
                }
                Message::Block(blocks) => {
                    info!("Got {} new blocks", blocks.len());
                }
            }
        }
    }
}
