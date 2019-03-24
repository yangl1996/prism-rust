use std::sync::{mpsc, Mutex, Arc};
use super::message;
use super::peer;
use std::thread;

pub struct Context {
    msg_chan: Arc<Mutex<mpsc::Receiver<(message::Message, peer::Handle)>>>,
    num_worker: usize,
}

pub fn new(num_worker: usize, msg_src: mpsc::Receiver<(message::Message, peer::Handle)>) -> Context {
    let ctx = Context {
        msg_chan: Arc::new(Mutex::new(msg_src)),
        num_worker: num_worker,
    };
    return ctx;
}

fn worker_loop(msg_chan: Arc<Mutex<mpsc::Receiver<(message::Message, peer::Handle)>>>) {
    loop {
        let msg = msg_chan.lock().unwrap().recv().unwrap();
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
