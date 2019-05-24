use crate::handler::new_transaction;
use crate::miner::memory_pool::MemoryPool;
use crate::network::server::Handle as ServerHandle;
use crate::transaction;
use crate::wallet::Wallet;
use log::{debug, error, info, trace};
use rand::Rng;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time;

pub enum ControlSignal {
    Start,
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
    pub fn new(
        wallet: &Arc<Wallet>,
        server: &ServerHandle,
        mempool: &Arc<Mutex<MemoryPool>>,
    ) -> (Self, mpsc::Sender<ControlSignal>) {
        let (tx, rx) = mpsc::channel();
        let instance = Self {
            wallet: Arc::clone(wallet),
            server: server.clone(),
            mempool: Arc::clone(mempool),
            control_chan: rx,
            arrival_distribution: ArrivalDistribution::Uniform(UniformArrival { interval: 100 }),
            value_distribution: ValueDistribution::Uniform(UniformValue { min: 50, max: 100 }),
            state: State::Paused,
        };
        return (instance, tx);
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Start => {
                self.state = State::Run;
                info!("Transaction generator started");
            }
            ControlSignal::Stop => {
                self.state = State::Paused;
                info!("Transaction generator paused");
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
            let addr = self.wallet.addresses().unwrap()[0];
            loop {
                // check the current state and try to receive control message
                match self.state {
                    State::Run => match self.control_chan.try_recv() {
                        Ok(signal) => {
                            self.handle_control_signal(signal);
                            continue;
                        }
                        Err(mpsc::TryRecvError::Empty) => {}
                        Err(mpsc::TryRecvError::Disconnected) => {
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
                let value: u64 = match &self.value_distribution {
                    ValueDistribution::Uniform(d) => rng.gen_range(d.min, d.max),
                };
                let transaction = self.wallet.create_transaction(addr, value);
                match transaction {
                    Ok(t) => {
                        new_transaction(t, &self.mempool, &self.server);
                    }
                    Err(e) => {
                        trace!("Failed to generate transaction: {}", e);
                    }
                };
                let interval: u64 = match &self.arrival_distribution {
                    ArrivalDistribution::Uniform(d) => d.interval,
                };
                let interval = time::Duration::from_millis(interval);
                thread::sleep(interval);
            }
        });
        info!("Transaction generator initialized into paused mode");
    }
}
