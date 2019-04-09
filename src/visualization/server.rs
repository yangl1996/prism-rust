use super::dump::dump_blockchain;
use crate::blockchain::BlockChain;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;

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
                let allow_all_origin =
                    tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..])
                        .unwrap();
                let resp = Response::from_string(dump_blockchain(&server.chain.lock().unwrap()))
                    .with_header(allow_all_origin);
                req.respond(resp);
            }
        });
    }
}
