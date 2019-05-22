mod transaction_generator;

use crate::wallet::Wallet;
use crate::network::server::Handle as ServerHandle;
use crate::miner::memory_pool::MemoryPool;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;
use url::Url;

pub struct Server {
    transaction_generator_handle: mpsc::Sender<transaction_generator::ControlSignal>,
    handle: HTTPServer,
}

impl Server {
    pub fn start(addr: std::net::SocketAddr, wallet: &Arc<Wallet>, server: &ServerHandle, mempool: &Arc<Mutex<MemoryPool>>) {
        // TODO: make it a separate API
        wallet.generate_keypair().unwrap();

        let (transaction_generator_sender, transaction_generator_receiver) = mpsc::channel();
        let transaction_generator = transaction_generator::TransactionGenerator::new(wallet, server, mempool, transaction_generator_receiver);
        transaction_generator.start();

        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle: handle,
            transaction_generator_handle: transaction_generator_sender,
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let transaction_generator_handle = server.transaction_generator_handle.clone();
                thread::spawn(move || {
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = base_url.join(req.url()).unwrap();
                    match url.path() {
                        "/transaction-generator/start" => {
                            let control_signal = transaction_generator::ControlSignal::Start;
                            transaction_generator_handle.send(control_signal).unwrap();
                            let resp = Response::from_string("success");
                            req.respond(resp).unwrap();
                        }
                        "/transaction-generator/stop" => {
                            let control_signal = transaction_generator::ControlSignal::Stop;
                            transaction_generator_handle.send(control_signal).unwrap();
                            let resp = Response::from_string("success");
                            req.respond(resp).unwrap();
                        }
                        _ => {
                        }
                    }
                });
            }
        });
    }
}
