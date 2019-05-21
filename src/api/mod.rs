mod transaction_generator;

use crate::wallet::Wallet;
use crate::network::server::Handle as ServerHandle;
use crate::miner::memory_pool::MemoryPool;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;

pub struct Server {
    transaction_generator_handle: mpsc::Sender<transaction_generator::ControlSignal>,
    handle: HTTPServer,
}

impl Server {
    pub fn start(addr: std::net::SocketAddr, wallet: &Arc<Wallet>, server: &ServerHandle, mempool: &Arc<Mutex<MemoryPool>>) {
        wallet.generate_keypair().unwrap();

        let (transaction_generator_sender, transaction_generator_receiver) = mpsc::channel();
        let mut transaction_generator = transaction_generator::TransactionGenerator::new(wallet, server, mempool, transaction_generator_receiver);
        transaction_generator.start();

        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle: handle,
            transaction_generator_handle: transaction_generator_sender,
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                thread::spawn(move || {
                });
            }
        });
    }
}
