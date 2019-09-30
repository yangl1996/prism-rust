use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::handler::new_transaction;
use crate::miner::memory_pool::MemoryPool;
use crate::network::server::Handle as ServerHandle;

use crate::wallet::Wallet;
use crossbeam::channel;
use log::{info, trace};
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

pub enum ControlSignal {
    Start(u64),
    Step(u64),
    Stop,
    SetArrivalDistribution(ArrivalDistribution),
    SetValueDistribution(ValueDistribution),
}

pub enum ArrivalDistribution {
    Uniform(UniformArrival),
}

pub struct UniformArrival {
    pub interval: u64, // ms
}

pub enum ValueDistribution {
    Uniform(UniformValue),
}

pub struct UniformValue {
    pub min: u64,
    pub max: u64,
}

enum State {
    Continuous(u64),
    Paused,
    Step(u64),
}

pub struct TransactionGenerator {
    wallet: Arc<Wallet>,
    server: ServerHandle,
    mempool: Arc<Mutex<MemoryPool>>,
    control_chan: channel::Receiver<ControlSignal>,
    arrival_distribution: ArrivalDistribution,
    value_distribution: ValueDistribution,
    state: State,
}

impl TransactionGenerator {
    pub fn new(
        wallet: &Arc<Wallet>,
        server: &ServerHandle,
        mempool: &Arc<Mutex<MemoryPool>>,
    ) -> (Self, channel::Sender<ControlSignal>) {
        let (tx, rx) = channel::unbounded();
        let instance = Self {
            wallet: Arc::clone(wallet),
            server: server.clone(),
            mempool: Arc::clone(mempool),
            control_chan: rx,
            arrival_distribution: ArrivalDistribution::Uniform(UniformArrival { interval: 100 }),
            value_distribution: ValueDistribution::Uniform(UniformValue { min: 50, max: 100 }),
            state: State::Paused,
        };
        (instance, tx)
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Start(t) => {
                self.state = State::Continuous(t);
                info!("Transaction generator started");
            }
            ControlSignal::Stop => {
                self.state = State::Paused;
                info!("Transaction generator paused");
            }
            ControlSignal::Step(num) => {
                self.state = State::Step(num);
                info!(
                    "Transaction generator started to generate {} transactions",
                    num
                );
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
            let mut prev_coin = None;
            loop {
                let tx_gen_start = time::Instant::now();
                // check the current state and try to receive control message
                match self.state {
                    State::Continuous(_) | State::Step(_) => match self.control_chan.try_recv() {
                        Ok(signal) => {
                            self.handle_control_signal(signal);
                            continue;
                        }
                        Err(channel::TryRecvError::Empty) => {}
                        Err(channel::TryRecvError::Disconnected) => {
                            panic!("Transaction generator control channel detached")
                        }
                    },
                    State::Paused => {
                        // block until we get a signal
                        let signal = self.control_chan.recv().unwrap();
                        self.handle_control_signal(signal);
                        continue;
                    }
                }
                // check whether the mempool is already full
                if let State::Continuous(throttle) = self.state {
                    if self.mempool.lock().unwrap().len() as u64 >= throttle {
                        // if the mempool is full, just skip this transaction
                        let interval: u64 = match &self.arrival_distribution {
                            ArrivalDistribution::Uniform(d) => d.interval,
                        };
                        let interval = time::Duration::from_micros(interval);
                        thread::sleep(interval);
                        continue;
                    }
                }
                let value: u64 = match &self.value_distribution {
                    ValueDistribution::Uniform(d) => {
                        if d.min == d.max {
                            d.min
                        } else {
                            rng.gen_range(d.min, d.max)
                        }
                    }
                };
                let transaction = self.wallet.create_transaction(addr, value, prev_coin);
                PERFORMANCE_COUNTER.record_generate_transaction(&transaction);
                match transaction {
                    Ok(t) => {
                        prev_coin = Some(t.input.last().unwrap().coin);
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
                        prev_coin = None;
                    }
                };
                let interval: u64 = match &self.arrival_distribution {
                    ArrivalDistribution::Uniform(d) => d.interval,
                };
                let interval = time::Duration::from_micros(interval);
                let time_spent = time::Instant::now().duration_since(tx_gen_start);
                let interval = {
                    if interval > time_spent {
                        interval - time_spent
                    } else {
                        time::Duration::new(0, 0)
                    }
                };
                thread::sleep(interval);
            }
        });
        info!("Transaction generator initialized into paused mode");
    }
}
