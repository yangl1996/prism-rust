use super::message;
use super::peer::{self, ReadResult, WriteResult};
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crossbeam::channel as cbchannel;
use log::{debug, error, info, trace, warn};
use mio::{self, net};
use mio_extras::channel;
use std::sync::mpsc;
use std::thread;

const MAX_INCOMING_CLIENT: usize = 256;
const MAX_EVENT: usize = 1024;

pub fn new(
    addr: std::net::SocketAddr,
    msg_sink: cbchannel::Sender<(Vec<u8>, peer::Handle)>,
) -> std::io::Result<(Context, Handle)> {
    let (control_signal_sender, control_signal_receiver) = channel::channel();
    let handle = Handle {
        control_chan: control_signal_sender,
    };
    let ctx = Context {
        peers: slab::Slab::new(),
        peer_list: vec![],
        addr,
        poll: mio::Poll::new()?,
        control_chan: control_signal_receiver,
        new_msg_chan: msg_sink,
        _handle: handle.clone(),
    };
    Ok((ctx, handle))
}

pub struct Context {
    peers: slab::Slab<peer::Context>,
    peer_list: Vec<usize>,
    addr: std::net::SocketAddr,
    poll: mio::Poll,
    control_chan: channel::Receiver<ControlSignal>,
    new_msg_chan: cbchannel::Sender<(Vec<u8>, peer::Handle)>,
    _handle: Handle,
}

impl Context {
    /// Start a new server context.
    pub fn start(mut self) -> std::io::Result<()> {
        thread::spawn(move || {
            self.listen().unwrap_or_else(|e| {
                error!("P2P server error: {}", e);
            });
        });
        Ok(())
    }

    /// Register a TCP stream in the event loop, and initialize peer context.
    fn register(
        &mut self,
        stream: net::TcpStream,
        direction: peer::Direction,
    ) -> std::io::Result<peer::Handle> {
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
        let (ctx, handle) = peer::new(stream, direction)?;

        // register the writer queue
        self.poll.register(
            &ctx.writer.queue,
            writer_token,
            mio::Ready::readable(),
            mio::PollOpt::edge() | mio::PollOpt::oneshot(),
        )?;

        // insert the context and return the handle
        vacant.insert(ctx);
        // record the key of this peer
        self.peer_list.push(key);
        trace!("Registering peer with event token={}", key);
        Ok(handle)
    }

    /// Connect to a peer, and register this peer
    fn connect(&mut self, addr: &std::net::SocketAddr) -> std::io::Result<peer::Handle> {
        // we need to estabilsh a stdlib tcp stream, since we need it to block
        debug!("Establishing connection to peer {}", addr);
        let stream = std::net::TcpStream::connect(addr)?;
        let mio_stream = net::TcpStream::from_stream(stream)?;
        self.register(mio_stream, peer::Direction::Outgoing)
    }

    /// Accept an incoming peer and register it
    fn accept(
        &mut self,
        stream: net::TcpStream,
        addr: std::net::SocketAddr,
    ) -> std::io::Result<()> {
        debug!("New incoming connection from {}", addr);
        match self.register(stream, peer::Direction::Incoming) {
            Ok(_) => {
                info!("Connected to incoming peer {}", addr);
            }
            Err(e) => {
                error!("Error initializing incoming peer {}: {}", addr, e);
            }
        }
        Ok(())
    }

    fn process_control(&mut self, req: ControlSignal) -> std::io::Result<()> {
        match req {
            ControlSignal::ConnectNewPeer(req) => {
                trace!("Processing ConnectNewPeer command");
                let handle = self.connect(&req.addr);
                req.result_chan.send(handle).unwrap();
            }
            ControlSignal::BroadcastMessage(msg) => {
                trace!("Processing BroadcastMessage command");
                for peer_id in &self.peer_list {
                    self.peers[*peer_id].handle.write(msg.clone());
                }
            }
        }
        Ok(())
    }

    fn register_write_interest(&mut self, peer_id: usize) -> std::io::Result<()> {
        trace!("Registering socket write interest for peer {}", peer_id);
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
        Ok(())
    }

    fn process_readable(&mut self, peer_id: usize) -> std::io::Result<()> {
        // we are using edge-triggered events, loop until block
        let peer = &mut self.peers[peer_id];
        loop {
            match peer.reader.read() {
                Ok(ReadResult::EOF) => {
                    // EOF, remove it from the connections set
                    info!("Peer {} dropped connection", peer.addr);
                    self.peers.remove(peer_id);
                    let index = self.peer_list.iter().position(|&x| x == peer_id).unwrap();
                    self.peer_list.swap_remove(index);
                    break;
                }
                Ok(ReadResult::Continue) => {
                    trace!("Peer {} reading continue", peer_id);
                    // no full message has been received
                    continue;
                }
                Ok(ReadResult::Message(m)) => {
                    trace!("Peer {} yield message", peer_id);
                    // we just received a full message
                    self.new_msg_chan.send((m, peer.handle.clone())).unwrap();
                    PERFORMANCE_COUNTER.record_receive_message();
                    continue;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        trace!("Peer {} finished reading", peer_id);
                        // socket is not ready anymore, stop reading
                        break;
                    } else {
                        warn!("Error reading peer {}, disconnecting: {}", peer.addr, e);
                        self.peers.remove(peer_id);
                        let index = self.peer_list.iter().position(|&x| x == peer_id).unwrap();
                        self.peer_list.swap_remove(index);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn process_writable(&mut self, peer_id: usize) -> std::io::Result<()> {
        let peer = &mut self.peers[peer_id];
        match peer.writer.write() {
            Ok(WriteResult::Complete) => {
                trace!("Peer {} outgoing queue drained", peer_id);
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
            }
            Ok(WriteResult::EOF) => {
                // EOF, remove it from the connections set
                info!("Peer {} dropped connection", peer.addr);
                self.peers.remove(peer_id);
                let index = self.peer_list.iter().position(|&x| x == peer_id).unwrap();
                self.peer_list.swap_remove(index);
            }
            Ok(WriteResult::ChanClosed) => {
                // the channel is closed. no more writes.
                warn!("Peer {} outgoing queue closed", peer_id);
                let socket_token = mio::Token(peer_id * 2);
                self.poll.reregister(
                    &peer.stream,
                    socket_token,
                    mio::Ready::readable(),
                    mio::PollOpt::edge(),
                )?;
                self.poll.deregister(&peer.writer.queue)?;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    trace!("Peer {} finished writing", peer_id);
                // socket is not ready anymore, stop reading
                } else {
                    warn!("Error writing peer {}, disconnecting: {}", peer.addr, e);
                    self.peers.remove(peer_id);
                    let index = self.peer_list.iter().position(|&x| x == peer_id).unwrap();
                    self.peer_list.swap_remove(index);
                }
            }
        }
        Ok(())
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

        info!("P2P server listening at {}", server.local_addr()?);

        // initialize space for polled events
        let mut events = mio::Events::with_capacity(MAX_EVENT);

        loop {
            self.poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    CONTROL => {
                        trace!("Server control channel readable");
                        // process until we drain the control message queue
                        loop {
                            // get the new control singal from the channel
                            match self.control_chan.try_recv() {
                                Ok(req) => {
                                    self.process_control(req).unwrap();
                                }
                                Err(e) => match e {
                                    mpsc::TryRecvError::Empty => break,
                                    mpsc::TryRecvError::Disconnected => {
                                        warn!("P2P server dropped, disconnecting all peers");
                                        self.poll.deregister(&self.control_chan)?;
                                        break;
                                    }
                                },
                            }
                        }
                    }
                    INCOMING => {
                        trace!("P2P server listener readable");
                        // we have a new connection
                        // we are using edge-triggered events, loop until block
                        loop {
                            // accept the connection
                            match server.accept() {
                                Ok((stream, client_addr)) => {
                                    self.accept(stream, client_addr).unwrap();
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
                                    trace!("Peer {} readable", peer_id);
                                    if !self.peers.contains(peer_id) {
                                        continue;
                                    }
                                    self.process_readable(peer_id).unwrap();
                                }
                                if readiness.is_writable() {
                                    trace!("Peer {} writable", peer_id);
                                    if !self.peers.contains(peer_id) {
                                        continue;
                                    }
                                    self.process_writable(peer_id).unwrap();
                                }
                            }
                            1 => {
                                trace!("Peer {} outgoing queue readable", peer_id);
                                self.register_write_interest(peer_id)?;
                            }
                            _ => unreachable!(),
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
        let (sender, receiver) = cbchannel::unbounded();
        let request = ConnectRequest {
            addr,
            result_chan: sender,
        };
        self.control_chan
            .send(ControlSignal::ConnectNewPeer(request))
            .unwrap();
        receiver.recv().unwrap()
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
    result_chan: cbchannel::Sender<std::io::Result<peer::Handle>>,
}
