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
    SetArrivalDistribution(ArrivalDistribution),
    SetValueDistribution(ValueDistribution),
}

pub enum ArrivalDistribution {
    Uniform(UniformArrival),
}

pub enum ValueDistribution {
    Uniform(UniformValue),
}

pub struct UniformArrival {
    interval: u64   // ms
}

pub struct UniformValue {
    min: u64,
    max: u64,
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
    arrival_distribution: ArrivalDistribution,
    value_distribution: ValueDistribution,
    state: State,
}

impl TransactionGenerator {
    pub fn new(wallet: &Arc<Wallet>, server: &ServerHandle, mempool: &Arc<Mutex<MemoryPool>>, control_chan: mpsc::Receiver<ControlSignal>) -> Self {
        return Self {
            wallet: Arc::clone(wallet),
            server: server.clone(),
            mempool: Arc::clone(mempool),
            control_chan: control_chan,
            arrival_distribution: ArrivalDistribution::Uniform(
                UniformArrival {
                    interval: 100,
                }
            ),
            value_distribution: ValueDistribution::Uniform(
                UniformValue {
                    min: 50,
                    max: 100,
                }
            ),
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
            ControlSignal::SetArrivalDistribution(new) => {
                self.arrival_distribution = new;
            }
            ControlSignal::SetValueDistribution(new) => {
                self.value_distribution = new;
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
                let value: u64 = match &self.value_distribution {
                    ValueDistribution::Uniform(d) => {
                        rng.gen_range(d.min, d.max)
                    }
                };
                let transaction = self.wallet.create_transaction(addr, value);
                match transaction {
                    Ok(t) => {
                        new_transaction(t, &self.mempool, &self.server);
                    }
                    Err(_) => {}
                };
                let interval: u64 = match &self.arrival_distribution {
                    ArrivalDistribution::Uniform(d) => {
                        d.interval
                    }
                };
                let interval = time::Duration::from_millis(interval);
                thread::sleep(interval);
            }
        });
    }
}
