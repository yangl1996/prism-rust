use super::dump::dump_blockchain;
use crate::blockchain::BlockChain;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;

pub struct Server {
    chain: Arc<Mutex<BlockChain>>,
    handle: HTTPServer,
}

/// This macro serves the static file at the location `path` and attaches the content type `type`.
macro_rules! serve_static_file {
    ( $req:expr, $path:expr, $type:expr ) => {{
        let content_type = concat!("Content-Type: ", $type).parse::<Header>().unwrap();
        let cache_control = "Cache-Control: public, max-age=31536000"
            .parse::<Header>()
            .unwrap();
        let resp = Response::from_string(include_str!($path))
            .with_header(content_type)
            .with_header(cache_control);
        $req.respond(resp).unwrap();
    }};
}

/// This macro serves the string `src` and attaches the content type `type`. Before serving the
/// string, all occurrances of `SERVER_IP_ADDR` and `SERVER_PORT_NUMBER` in the string are replaced
/// with the server IP and port respectively.
macro_rules! serve_dynamic_file {
    ( $req:expr, $src:expr, $type:expr, $addr:expr ) => {{
        let source = $src
            .to_string()
            .replace("SERVER_IP_ADDR", &$addr.ip().to_string())
            .replace("SERVER_PORT_NUMBER", &$addr.port().to_string());
        let content_type = concat!("Content-Type: ", $type).parse::<Header>().unwrap();
        let cache_control = "Cache-Control: no-store".parse::<Header>().unwrap();
        let allow_all = "Access-Control-Allow-Origin: *".parse::<Header>().unwrap();
        let resp = Response::from_string(source)
            .with_header(content_type)
            .with_header(cache_control)
            .with_header(allow_all);
        $req.respond(resp).unwrap();
    }};
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
                thread::spawn(move || match req.url().trim_start_matches("/") {
                    "blockchain.json" => serve_dynamic_file!(
                        req,
                        dump_blockchain(&chain.lock().unwrap()),
                        "application/json",
                        addr
                    ),
                    "cytoscape.min.js" => {
                        serve_static_file!(req, "cytoscape.js", "application/javascript")
                    }
                    "dagre.min.js" => {
                        serve_static_file!(req, "dagre.min.js", "application/javascript")
                    }
                    "cytoscape-dagre.js" => {
                        serve_static_file!(req, "cytoscape-dagre.js", "application/javascript")
                    }
                    "bootstrap.min.css" => serve_static_file!(req, "bootstrap.min.css", "text/css"),
                    "blockchain_vis.js" => serve_dynamic_file!(
                        req,
                        include_str!("blockchain_vis.js"),
                        "application/javascript",
                        addr
                    ),
                    "visualize-blockchain" => serve_dynamic_file!(
                        req,
                        include_str!("blockchain_vis.html"),
                        "text/html",
                        addr
                    ),
                    "" => serve_dynamic_file!(req, include_str!("index.html"), "text/html", addr),
                    _ => {
                        let content_type = "Content-Type: text/html".parse::<Header>().unwrap();
                        let resp = Response::from_string(include_str!("404.html"))
                            .with_header(content_type)
                            .with_status_code(404);
                        req.respond(resp);
                    }
                });
            }
        });
    }
}
