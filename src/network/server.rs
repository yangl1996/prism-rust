use super::message;
use super::peer::{self, ReadResult, WriteResult};
use log::{debug, error, info, warn};
use mio::{self, net};
use mio_extras::channel;
use std::sync::mpsc;
use std::thread;

const MAX_INCOMING_CLIENT: usize = 256;
const MAX_EVENT: usize = 1024;

pub fn new(
    addr: std::net::SocketAddr,
    msg_sink: mpsc::Sender<(message::Message, peer::Handle)>,
) -> std::io::Result<(Context, Handle)> {
    let (control_signal_sender, control_signal_receiver) = channel::channel();
    let ctx = Context {
        peers: slab::Slab::new(),
        addr: addr,
        poll: mio::Poll::new()?,
        control_chan: control_signal_receiver,
        new_msg_chan: msg_sink,
    };
    let handle = Handle {
        control_chan: control_signal_sender,
    };
    return Ok((ctx, handle));
}

pub struct Context {
    peers: slab::Slab<peer::Context>,
    addr: std::net::SocketAddr,
    poll: mio::Poll,
    control_chan: channel::Receiver<ControlSignal>,
    new_msg_chan: mpsc::Sender<(message::Message, peer::Handle)>,
}

impl Context {
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
    fn register(&mut self, stream: net::TcpStream) -> std::io::Result<peer::Handle> {
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

        // set two tokens, one for socket and one for write queue
        let socket_token = mio::Token(key * 2);
        let writer_token = mio::Token(key * 2 + 1);

        // register the new connection
        self.poll.register(
            &stream,
            socket_token,
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;
        let (ctx, handle) = peer::new(stream)?;

        // register the writer queue
        self.poll.register(
            &ctx.writer.queue,
            writer_token,
            mio::Ready::readable(),
            mio::PollOpt::edge() | mio::PollOpt::oneshot(),
        )?;

        // insert the context and return the handle
        vacant.insert(ctx);
        return Ok(handle);
    }

    /// Connect to a peer, and register this peer
    fn connect(&mut self, addr: &std::net::SocketAddr) -> std::io::Result<peer::Handle> {
        // we need to estabilsh a stdlib tcp stream, since we need it to block
        let stream = std::net::TcpStream::connect(addr)?;
        let mio_stream = net::TcpStream::from_stream(stream)?;
        return self.register(mio_stream);
    }

    /// The main event loop of the server.
    fn listen(&mut self) -> std::io::Result<()> {
        // bind server to passed addr and register to the poll
        let server = net::TcpListener::bind(&self.addr)?;

        // token for new incoming connection
        const INCOMING: mio::Token = mio::Token(std::usize::MAX - 1);
        self.poll.register(
            &server,
            INCOMING,
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;

        // token for new control signal from the handle
        const CONTROL: mio::Token = mio::Token(std::usize::MAX - 2);
        self.poll.register(
            &self.control_chan,
            CONTROL,
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
                    CONTROL => {
                        // we have a new control signal
                        loop {
                            // get the new control singal from the channel
                            match self.control_chan.try_recv() {
                                Ok(req) => {
                                    match req {
                                        ControlSignal::ConnectNewPeer(req) => {
                                            let handle = self.connect(&req.addr);
                                            req.result_chan.send(handle).unwrap();
                                        }
                                        ControlSignal::BroadcastMessage(msg) => {
                                            // TODO: slab iteration is slow. use a hashset to keep
                                            // the id of live connections
                                            for peer in self.peers.iter() {
                                                peer.1.handle.write(msg.clone());
                                            }
                                        }
                                    }
                                }
                                Err(e) => match e {
                                    mpsc::TryRecvError::Empty => break,
                                    mpsc::TryRecvError::Disconnected => {
                                        self.poll.deregister(&self.control_chan)?;
                                        break;
                                    }
                                },
                            }
                        }
                    }
                    INCOMING => {
                        // we have a new connection
                        // we are using edge-triggered events, loop until block
                        loop {
                            // accept the connection
                            match server.accept() {
                                Ok((stream, client_addr)) => {
                                    debug!("New incoming connection from {}", client_addr);
                                    match self.register(stream) {
                                        Ok(_) => {
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
                        // peer id (the index in the peers list) is token_id/2
                        let peer_id = token_id >> 1;
                        // if the token_id is odd, it's new write request, else it's socket
                        match token_id & 0x01 {
                            0 => {
                                let readiness = event.readiness();
                                if readiness.is_readable() {
                                    // we are using edge-triggered events, loop until block
                                    let peer = &mut self.peers[peer_id];
                                    loop {
                                        match peer.reader.read() {
                                            Ok(ReadResult::EOF) => {
                                                // EOF, remove it from the connections set
                                                info!("Peer {} dropped connection", peer.addr);
                                                self.peers.remove(peer_id);
                                                break;
                                            }
                                            Ok(ReadResult::Continue) => {
                                                continue;
                                            }
                                            Ok(ReadResult::Message(m)) => {
                                                self.new_msg_chan
                                                    .send((m, peer.handle.clone()))
                                                    .unwrap();
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
                                                    self.peers.remove(peer_id);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                if readiness.is_writable() {
                                    let peer = &mut self.peers[peer_id];
                                    match peer.writer.write() {
                                        Ok(WriteResult::Complete) => {
                                            // we wrote everything in the write queue
                                            let socket_token = mio::Token(peer_id * 2);
                                            let writer_token = mio::Token(peer_id * 2 + 1);
                                            // we've done writing. no longer interested.
                                            self.poll.reregister(
                                                &peer.stream,
                                                socket_token,
                                                mio::Ready::readable(),
                                                mio::PollOpt::edge(),
                                            )?;
                                            // we're interested in write queue again.
                                            self.poll.reregister(
                                                &peer.writer.queue,
                                                writer_token,
                                                mio::Ready::readable(),
                                                mio::PollOpt::edge() | mio::PollOpt::oneshot(),
                                            )?;
                                            continue;
                                        }
                                        Ok(WriteResult::EOF) => {
                                            // EOF, remove it from the connections set
                                            info!("Peer {} dropped connection", peer.addr);
                                            self.peers.remove(peer_id);
                                            continue; // continue event loop
                                        }
                                        Ok(WriteResult::ChanClosed) => {
                                            // the channel is closed. no more writes.
                                            let socket_token = mio::Token(peer_id * 2);
                                            self.poll.reregister(
                                                &peer.stream,
                                                socket_token,
                                                mio::Ready::readable(),
                                                mio::PollOpt::edge(),
                                            )?;
                                            self.poll.deregister(&peer.writer.queue)?;
                                            continue;
                                        }
                                        Err(e) => {
                                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                                // socket is not ready anymore, stop reading
                                                continue; // continue event loop
                                            } else {
                                                warn!(
                                                    "Error writing peer {}, disconnecting: {}",
                                                    peer.addr, e
                                                );
                                                // TODO: we did not shutdown the stream. Cool?
                                                self.peers.remove(peer_id);
                                                continue; // continue event loop
                                            }
                                        }
                                    }
                                }
                            }
                            1 => {
                                let peer = &mut self.peers[peer_id];
                                // we have stuff to write at the writer queue
                                let socket_token = mio::Token(peer_id * 2);
                                // register for writable event
                                self.poll.reregister(
                                    &peer.stream,
                                    socket_token,
                                    mio::Ready::readable() | mio::Ready::writable(),
                                    mio::PollOpt::edge(),
                                )?;
                            }
                            _ => unimplemented!(),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Handle {
    control_chan: channel::Sender<ControlSignal>,
}

impl Handle {
    pub fn connect(&self, addr: std::net::SocketAddr) -> std::io::Result<peer::Handle> {
        let (sender, receiver) = mpsc::channel();
        let request = ConnectRequest {
            addr: addr,
            result_chan: sender,
        };
        self.control_chan
            .send(ControlSignal::ConnectNewPeer(request))
            .unwrap();
        return receiver.recv().unwrap();
    }

    pub fn broadcast(&self, msg: message::Message) {
        self.control_chan
            .send(ControlSignal::BroadcastMessage(msg))
            .unwrap();
    }
}

enum ControlSignal {
    ConnectNewPeer(ConnectRequest),
    BroadcastMessage(message::Message),
}

struct ConnectRequest {
    addr: std::net::SocketAddr,
    result_chan: mpsc::Sender<std::io::Result<peer::Handle>>,
}
