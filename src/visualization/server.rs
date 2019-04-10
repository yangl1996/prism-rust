use crate::blockchain::BlockChain;
use tiny_http::Server as HTTPServer;
use tiny_http::Response;
use tiny_http::Header;
use std::sync::{Arc, Mutex};
use std::thread;
use super::dump::dump_blockchain;
use super::template::BLOCKCHAIN_VISUALIZATION;
use super::js::CYTOSCAPE;

pub struct Server {
    chain: Arc<Mutex<BlockChain>>,
    handle: HTTPServer,
}

impl Server {
    pub fn start(addr: std::net::SocketAddr, chain: Arc<Mutex<BlockChain>>) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            chain: chain,
            handle: handle,
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                match req.url().trim_start_matches("/") {
                    "blockchain.json" => {
                        let allow_all_origin = "Access-Control-Allow-Origin: *".parse::<Header>().unwrap();
                        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
                        let cache_control = "Cache-Control: no-store".parse::<Header>().unwrap();
                        let resp = Response::from_string(dump_blockchain(&server.chain.lock().unwrap()))
                            .with_header(allow_all_origin)
                            .with_header(cache_control)
                            .with_header(content_type);
                        req.respond(resp);
                    },
                    "cytoscape.min.js" => {
                        let content_type = "Content-Type: application/javascript".parse::<Header>().unwrap();
                        let cache_control = "Cache-Control: public, max-age=31536000".parse::<Header>().unwrap();
                        let resp = Response::from_string(CYTOSCAPE)
                            .with_header(content_type)
                            .with_header(cache_control);
                        req.respond(resp);
                    }
                    _ => {
                        let vis_page = BLOCKCHAIN_VISUALIZATION.to_string()
                            .replace("SERVER_IP_ADDR", &addr.ip().to_string())
                            .replace("SERVER_PORT_NUMBER", &addr.port().to_string());
                        let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
                        let resp = Response::from_string(vis_page)
                            .with_header(content_type);
                        req.respond(resp);
                    }
                }
            }
        });
    }
}
