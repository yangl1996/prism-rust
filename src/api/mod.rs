use crate::blockchain::BlockChain;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::experiment::transaction_generator;
use crate::miner::memory_pool::MemoryPool;
use crate::miner::Handle as MinerHandle;
use crate::network::server::Handle as ServerHandle;
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use crossbeam::channel;
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;
use url::Url;

pub struct Server {
    transaction_generator_handle: crossbeam::Sender<transaction_generator::ControlSignal>,
    handle: HTTPServer,
    miner: MinerHandle,
    wallet: Arc<Wallet>,
    utxodb: Arc<UtxoDatabase>,
    blockchain: Arc<BlockChain>,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct WalletBalanceResponse {
    balance: u64,
}

#[derive(Serialize)]
struct UtxoSnapshotResponse {
    checksum: String,
}

#[derive(Serialize)]
struct BlockchainSnapshotResponse {
    leaders: Vec<String>,
}

macro_rules! respond_result {
    ( $req:expr, $success:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let payload = ApiResponse {
            success: $success,
            message: $message.to_string(),
        };
        let resp = Response::from_string(serde_json::to_string_pretty(&payload).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

macro_rules! respond_json {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let resp = Response::from_string(serde_json::to_string_pretty(&$message).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        wallet: &Arc<Wallet>,
        blockchain: &Arc<BlockChain>,
        utxodb: &Arc<UtxoDatabase>,
        server: &ServerHandle,
        miner: &MinerHandle,
        mempool: &Arc<Mutex<MemoryPool>>,
        txgen_control_chan: crossbeam::Sender<transaction_generator::ControlSignal>,
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle: handle,
            transaction_generator_handle: txgen_control_chan,
            miner: miner.clone(),
            wallet: Arc::clone(wallet),
            utxodb: Arc::clone(utxodb),
            blockchain: Arc::clone(blockchain),
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let transaction_generator_handle = server.transaction_generator_handle.clone();
                let miner = server.miner.clone();
                let wallet = Arc::clone(&server.wallet);
                let utxodb = Arc::clone(&server.utxodb);
                let blockchain = Arc::clone(&server.blockchain);
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(e) => {
                            respond_result!(req, false, format!("error parsing url: {}", e));
                            return;
                        }
                    };
                    match url.path() {
                        "/blockchain/snapshot" => {
                            let leaders = blockchain.proposer_leaders().unwrap();
                            let leader_hash_strings: Vec<String> =
                                leaders.iter().map(|x| x.to_string()).collect();
                            let resp = BlockchainSnapshotResponse {
                                leaders: leader_hash_strings,
                            };
                            respond_json!(req, resp);
                        }
                        "/utxo/snapshot" => {
                            let checksum = utxodb.snapshot().unwrap();
                            let resp = UtxoSnapshotResponse {
                                checksum: base64::encode(&checksum),
                            };
                            respond_json!(req, resp);
                        }
                        "/wallet/balance" => {
                            let resp = WalletBalanceResponse {
                                balance: wallet.balance().unwrap(),
                            };
                            respond_json!(req, resp);
                        }
                        "/miner/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let delta = match params.get("delta") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing delta");
                                    return;
                                }
                            };
                            let delta = match delta.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing delta: {}", e)
                                    );
                                    return;
                                }
                            };
                            let lazy = match params.get("lazy") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing lazy switch");
                                    return;
                                }
                            };
                            let lazy = match lazy.parse::<bool>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing lazy switch: {}", e)
                                    );
                                    return;
                                }
                            };
                            miner.start(delta, lazy);
                            respond_result!(req, true, "ok");
                        }
                        "/miner/step" => {
                            miner.step();
                            respond_result!(req, true, "ok");
                        }
                        "/telematics/snapshot" => {
                            respond_json!(req, PERFORMANCE_COUNTER.snapshot());
                        }
                        "/transaction-generator/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let throttle = match params.get("throttle") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing throttle");
                                    return;
                                }
                            };
                            let throttle = match throttle.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing throttle: {}", e)
                                    );
                                    return;
                                }
                            };
                            let control_signal =
                                transaction_generator::ControlSignal::Start(throttle);
                            match transaction_generator_handle.send(control_signal) {
                                Ok(()) => respond_result!(req, true, "ok"),
                                Err(e) => respond_result!(
                                    req,
                                    false,
                                    format!(
                                        "error sending control signal to transaction generator: {}",
                                        e
                                    )
                                ),
                            }
                        }
                        "/transaction-generator/stop" => {
                            let control_signal = transaction_generator::ControlSignal::Stop;
                            match transaction_generator_handle.send(control_signal) {
                                Ok(()) => respond_result!(req, true, "ok"),
                                Err(e) => respond_result!(
                                    req,
                                    false,
                                    format!(
                                        "error sending control signal to transaction generator: {}",
                                        e
                                    )
                                ),
                            }
                        }
                        "/transaction-generator/step" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let step_count = match params.get("count") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing step count");
                                    return;
                                }
                            };
                            let step_count = match step_count.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing step count: {}", e)
                                    );
                                    return;
                                }
                            };
                            let control_signal =
                                transaction_generator::ControlSignal::Step(step_count);
                            match transaction_generator_handle.send(control_signal) {
                                Ok(()) => respond_result!(req, true, "ok"),
                                Err(e) => respond_result!(
                                    req,
                                    false,
                                    format!(
                                        "error sending control signal to transaction generator: {}",
                                        e
                                    )
                                ),
                            }
                        }
                        "/transaction-generator/set-arrival-distribution" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let distribution = match params.get("distribution") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing distribution");
                                    return;
                                }
                            };
                            let distribution = match distribution.as_ref() {
                                "uniform" => {
                                    let interval = match params.get("interval") {
                                        Some(v) => match v.parse::<u64>() {
                                            Ok(v) => v,
                                            Err(e) => {
                                                respond_result!(
                                                    req,
                                                    false,
                                                    format!("error parsing interval: {}", e)
                                                );
                                                return;
                                            }
                                        },
                                        None => {
                                            respond_result!(req, false, "missing interval");
                                            return;
                                        }
                                    };
                                    transaction_generator::ArrivalDistribution::Uniform(
                                        transaction_generator::UniformArrival {
                                            interval: interval,
                                        },
                                    )
                                }
                                d => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("invalid distribution: {}", d)
                                    );
                                    return;
                                }
                            };
                            let control_signal =
                                transaction_generator::ControlSignal::SetArrivalDistribution(
                                    distribution,
                                );
                            match transaction_generator_handle.send(control_signal) {
                                Ok(()) => respond_result!(req, true, "ok"),
                                Err(e) => respond_result!(
                                    req,
                                    false,
                                    format!(
                                        "error sending control signal to transaction generator: {}",
                                        e
                                    )
                                ),
                            }
                        }
                        "/transaction-generator/set-value-distribution" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let distribution = match params.get("distribution") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing distribution");
                                    return;
                                }
                            };
                            let distribution = match distribution.as_ref() {
                                "uniform" => {
                                    let min = match params.get("min") {
                                        Some(v) => match v.parse::<u64>() {
                                            Ok(v) => v,
                                            Err(e) => {
                                                respond_result!(
                                                    req,
                                                    false,
                                                    format!("error parsing min: {}", e)
                                                );
                                                return;
                                            }
                                        },
                                        None => {
                                            respond_result!(req, false, "missing min");
                                            return;
                                        }
                                    };
                                    let max = match params.get("max") {
                                        Some(v) => match v.parse::<u64>() {
                                            Ok(v) => v,
                                            Err(e) => {
                                                respond_result!(
                                                    req,
                                                    false,
                                                    format!("error parsing max: {}", e)
                                                );
                                                return;
                                            }
                                        },
                                        None => {
                                            respond_result!(req, false, "missing max");
                                            return;
                                        }
                                    };
                                    if min > max {
                                        respond_result!(
                                            req,
                                            false,
                                            format!("min value is bigger than max value")
                                        );
                                        return;
                                    }
                                    transaction_generator::ValueDistribution::Uniform(
                                        transaction_generator::UniformValue { min: min, max: max },
                                    )
                                }
                                d => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("invalid distribution: {}", d)
                                    );
                                    return;
                                }
                            };
                            let control_signal =
                                transaction_generator::ControlSignal::SetValueDistribution(
                                    distribution,
                                );
                            match transaction_generator_handle.send(control_signal) {
                                Ok(()) => respond_result!(req, true, "ok"),
                                Err(e) => respond_result!(
                                    req,
                                    false,
                                    format!(
                                        "error sending control signal to transaction generator: {}",
                                        e
                                    )
                                ),
                            }
                        }
                        _ => {
                            let content_type =
                                "Content-Type: application/json".parse::<Header>().unwrap();
                            let payload = ApiResponse {
                                success: false,
                                message: "endpoint not found".to_string(),
                            };
                            let resp = Response::from_string(
                                serde_json::to_string_pretty(&payload).unwrap(),
                            )
                            .with_header(content_type)
                            .with_status_code(404);
                            req.respond(resp).unwrap();
                        }
                    }
                });
            }
        });
        info!("API server listening at {}", &addr);
    }
}
