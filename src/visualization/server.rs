use crate::blockchain::BlockChain;
use tiny_http::Server as HTTPServer;
use tiny_http::Response;
use std::sync::{Arc, Mutex};
use std::thread;
use super::dump::dump_blockchain;

pub struct Server {
    chain: Arc<Mutex<BlockChain>>,
    handle: HTTPServer,
}

impl Server {
    pub fn start(addr: std::net::SocketAddr, chain: Arc<Mutex<BlockChain>>) {
        let handle = HTTPServer::http(addr).unwrap();
        let server = Self {
            chain: chain,
            handle: handle,
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let resp = Response::from_string(dump_blockchain(&server.chain.lock().unwrap()));
                req.respond(resp);
            }
        });
        
    }
}
