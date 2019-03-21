use super::message;
use super::peer::{self, ReadResult};
use byteorder::{BigEndian, ByteOrder};
use log::{debug, error, info, trace, warn};
use mio::{self, net};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

const MAX_INCOMING_CLIENT: usize = 256;
const MAX_EVENT: usize = 1024;

pub struct Context {
    peers: slab::Slab<peer::Context>,
    addr: std::net::SocketAddr,
    poll: mio::Poll,
}

impl Context {
    /// Create a new server context.
    pub fn new(addr: std::net::SocketAddr) -> std::io::Result<Self> {
        let server = Self {
            peers: slab::Slab::new(),
            addr: addr,
            poll: mio::Poll::new()?,
        };
        return Ok(server);
    }

    /// Start a new server context.
    pub fn start(mut self) -> std::io::Result<()> {
        thread::spawn(move || {
            self.listen().unwrap_or_else(|e| {
                error!("Error occurred in P2P server: {}", e);
                return;
            });
        });
        return Ok(());
    }
    
    /// Register a TCP stream in the event loop, and initialize peer context.
    fn register(&mut self, stream: net::TcpStream) -> std::io::Result<()> {
        // get a new slot in the connection set
        let vacant = self.peers.vacant_entry();
        let key: usize = vacant.key();
        if key >= MAX_INCOMING_CLIENT {
            // too many connections
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "max peer reached, cannot accept new connections",
            ));
        }
        // register the new connection and insert
        self.poll.register(
            &stream,
            mio::Token(key),
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;
        let ctx = peer::Context::new(stream, mio::Token(key))?;
        vacant.insert(ctx);
        return Ok(());
    }

    /// The main event loop of the server.
    fn listen(&mut self) -> std::io::Result<()> {
        // bind server to passed addr and register to the poll
        let server = net::TcpListener::bind(&self.addr)?;

        // token for server new connection event
        const INCOMING: mio::Token = mio::Token(std::usize::MAX - 1); 
        self.poll.register(
            &server,
            INCOMING,
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;
        info!(
            "P2P server listening to incoming connections at {}",
            server.local_addr()?
        );

        // initialize space for polled events
        let mut events = mio::Events::with_capacity(MAX_EVENT);

        loop {
            self.poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    INCOMING => {
                        // we have a new connection
                        // we are using edge-triggered events, loop until block
                        loop {
                            // accept the connection
                            match server.accept() {
                                Ok((stream, client_addr)) => {
                                    debug!("New incoming connection from {}", client_addr);
                                    match self.register(stream) {
                                        Ok(()) => {
                                            info!("New incoming peer {}", client_addr);
                                        }
                                        Err(e) => {
                                            error!(
                                                "Error initializing incoming peer {}: {}",
                                                client_addr, e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock {
                                        // socket is not ready anymore, stop reading here
                                        break;
                                    } else {
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    }
                    mio::Token(token_id) => {
                        // one of the connected sockets is ready to read
                        let peer = &mut self.peers[token_id];
                        // we are using edge-triggered events, loop until block
                        loop {
                            match peer.read() {
                                Ok(ReadResult::EOF) => {
                                    // EOF, remove it from the connections set
                                    info!("Peer {} dropped connection", peer.addr);
                                    self.peers.remove(token_id);
                                    break;
                                }
                                Ok(ReadResult::Continue) => {
                                    continue;
                                }
                                Ok(ReadResult::Message(m)) => {
                                    info!("New message");
                                    continue;
                                }
                                Err(e) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock {
                                        // socket is not ready anymore, stop reading
                                        break;
                                    } else {
                                        warn!(
                                            "Error reading peer {}, disconnecting: {}",
                                            peer.addr, e
                                        );
                                        // TODO: we did not shutdown the stream. Cool?
                                        self.peers.remove(token_id);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/*
pub struct ServerHandler {
    pub addr: std::net::SocketAddr,
}

    /// Connect to a peer, and register this peer
    pub fn connect(&self, addr: &std::net::SocketAddr) -> std::io::Result<Arc<Peer>> {
        // we need to estabilsh a stdlib tcp stream, since we need it to block
        let stream = std::net::TcpStream::connect(addr)?;
        let mio_stream = net::TcpStream::from_stream(stream)?;
        return self.register_new_peer(mio_stream);
  }
  */
