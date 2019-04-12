use crate::blockchain::BlockChain;
use tiny_http::Server as HTTPServer;
use tiny_http::Response;
use tiny_http::Header;
use std::sync::{Arc, Mutex};
use std::thread;
use super::dump::dump_blockchain;

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
                let chain = Arc::clone(&server.chain);
                thread::spawn(move || {
                    match req.url().trim_start_matches("/") {
                        "blockchain.json" => {
                            let allow_all_origin = "Access-Control-Allow-Origin: *".parse::<Header>().unwrap();
                            let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
                            let cache_control = "Cache-Control: no-store".parse::<Header>().unwrap();
                            let resp = Response::from_string(dump_blockchain(&chain.lock().unwrap()))
                                .with_header(allow_all_origin)
                                .with_header(cache_control)
                                .with_header(content_type);
                            req.respond(resp);
                        },
                        "cytoscape.min.js" => {
                            let content_type = "Content-Type: application/javascript".parse::<Header>().unwrap();
                            let cache_control = "Cache-Control: public, max-age=31536000".parse::<Header>().unwrap();
                            let resp = Response::from_string(include_str!("cytoscape.js"))
                                .with_header(content_type)
                                .with_header(cache_control);
                            req.respond(resp);
                        },
                        "blockchain.html" => {
                            let vis_page = include_str!("blockchain.html").to_string()
                                .replace("SERVER_IP_ADDR", &addr.ip().to_string())
                                .replace("SERVER_PORT_NUMBER", &addr.port().to_string());
                            let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
                            let resp = Response::from_string(vis_page)
                                .with_header(content_type);
                            req.respond(resp);
                        }
                        "index.html" => {
                            let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
                            let resp = Response::from_string(include_str!("index.html"))
                                .with_header(content_type);
                            req.respond(resp);
                        }
                        "" => {
                            let redirect = "Location: /index.html".parse::<Header>().unwrap();
                            let resp = Response::empty(301)
                                .with_header(redirect);
                            req.respond(resp);
                        }
                        _ => {
                            let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
                            let resp = Response::from_string(include_str!("404.html"))
                                .with_header(content_type)
                                .with_status_code(404);
                            req.respond(resp);
                        }
                    }
                });
            }
        });
    }
}
