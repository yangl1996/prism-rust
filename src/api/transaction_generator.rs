use crate::miner::memory_pool::MemoryPool;
use crate::network::server::Handle as ServerHandle;
use crate::wallet::Wallet;
use crate::transaction;
use std::thread;
use std::time;
use std::sync::{Arc, Mutex, mpsc};
use rand::Rng;
use crate::handler::new_transaction;

pub enum ControlSignal {
    Start,
    Stop,
    SetInterval(u64),
    SetMaxSize(u64),
    SetMinSize(u64),
}

enum State {
    Run,
    Paused,
}

pub struct TransactionGenerator {
    wallet: Arc<Wallet>,
    server: ServerHandle,
    mempool: Arc<Mutex<MemoryPool>>,
    control_chan: mpsc::Receiver<ControlSignal>,
    interval: u64,  // ms
    max_size: u64,
    min_size: u64,
    state: State,
}

impl TransactionGenerator {
    pub fn new(wallet: &Arc<Wallet>, server: &ServerHandle, mempool: &Arc<Mutex<MemoryPool>>, control_chan: mpsc::Receiver<ControlSignal>) -> Self {
        return Self {
            wallet: Arc::clone(wallet),
            server: server.clone(),
            mempool: Arc::clone(mempool),
            control_chan: control_chan,
            interval: 100,
            max_size: 100,
            min_size: 50,
            state: State::Paused,
        };
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Start => {
                self.state = State::Run;
            }
            ControlSignal::Stop => {
                self.state = State::Paused;
            }
            ControlSignal::SetInterval(new) => {
                self.interval = new;
            }
            ControlSignal::SetMaxSize(new) => {
                self.max_size = new;
            }
            ControlSignal::SetMinSize(new) => {
                self.min_size = new;
            }
        }
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let addr = self.wallet.get_an_address().unwrap();
            loop {
                // check the current state and try to receive control message
                match self.state {
                    State::Run => {
                        match self.control_chan.try_recv() {
                            Ok(signal) => {
                                self.handle_control_signal(signal);
                                continue;
                            }
                            Err(mpsc::TryRecvError::Empty) => {}
                            Err(mpsc::TryRecvError::Disconnected) => panic!("Transaction generator control channel detached"),
                        }
                    }
                    State::Paused => {
                        // block until we get a signal
                        let signal = self.control_chan.recv().unwrap();
                        self.handle_control_signal(signal);
                        continue;
                    }
                }
                let value: u64 = rng.gen_range(self.min_size, self.max_size);
                let transaction = self.wallet.create_transaction(addr, value);
                match transaction {
                    Ok(t) => {
                        new_transaction(t, &self.mempool, &self.server);
                    }
                    Err(_) => {}
                };
                let interval = time::Duration::from_millis(self.interval);
                thread::sleep(interval);
            }
        });
    }
}
