use crate::miner::memory_pool::MemoryPool;
use crate::network::server::Handle as ServerHandle;
use crate::wallet::Wallet;
use crate::transaction;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use std::thread;
use std::time;
use std::sync::{Arc, Mutex, mpsc};
use rand::Rng;
use crate::handler::new_transaction;
use log::{debug, error, info, trace};

pub enum ControlSignal {
    Start,
    Step(u64),
    Stop,
    SetArrivalDistribution(ArrivalDistribution),
    SetValueDistribution(ValueDistribution),
}

pub enum ArrivalDistribution {
    Uniform(UniformArrival),
}

pub struct UniformArrival {
    pub interval: u64   // ms
}

pub enum ValueDistribution {
    Uniform(UniformValue),
}

pub struct UniformValue {
    pub min: u64,
    pub max: u64,
}

enum State {
    Continuous,
    Paused,
    Step(u64),
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
    pub fn new(wallet: &Arc<Wallet>, server: &ServerHandle, mempool: &Arc<Mutex<MemoryPool>>) -> (Self, mpsc::Sender<ControlSignal>) {
        let (tx, rx) = mpsc::channel();
        let instance = Self {
            wallet: Arc::clone(wallet),
            server: server.clone(),
            mempool: Arc::clone(mempool),
            control_chan: rx,
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
        return (instance, tx);
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Start => {
                self.state = State::Continuous;
                info!("Transaction generator started");
            }
            ControlSignal::Stop => {
                self.state = State::Paused;
                info!("Transaction generator paused");
            }
            ControlSignal::Step(num) => {
                self.state = State::Step(num);
                info!("Transaction generator started to generate {} transactions", num);
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
            // TODO: make it flexible
            let addr = self.wallet.addresses().unwrap()[0];
            loop {
                // check the current state and try to receive control message
                match self.state {
                    State::Continuous | State::Step(_) => {
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
                PERFORMANCE_COUNTER.record_generate_transaction(&transaction);
                match transaction {
                    Ok(t) => {
                        new_transaction(t, &self.mempool, &self.server);
                        // if we are in stepping mode, decrease the step count
                        if let State::Step(step_count) = self.state {
                            if step_count - 1 == 0 {
                                self.state = State::Paused;
                            } else {
                                self.state = State::Step(step_count - 1);
                            }
                        }
                    }
                    Err(e) => {
                        trace!("Failed to generate transaction: {}", e);
                    }
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
        info!("Transaction generator initialized into paused mode");
    }
}
