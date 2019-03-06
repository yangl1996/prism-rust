use super::message;
use byteorder::{BigEndian, ByteOrder};
use log::{debug, error, info, trace, warn};
use mio::{self, net};
use std::cell::UnsafeCell;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
use std::thread;

const MAX_INCOMING_CLIENT: usize = 256;
const MAX_EVENT: usize = 1024;

// PeerReaderCell is UnsafeCell with Send and Sync forcefully marked. It is used to
// enable interior mutability of Peer. We need it because we use RwLock to control
// peers, and can only obtain &Peer when obtaining a read lock on peers. Thus, we
// need to implement read() for &Peer, instead of &mut Peer. By manually ensuring
// that only one thread has access to the fields wrapped in PeerReaderCell, we don't
// have data race, and avoid the performance overhead of wrapping those states in
// Mutex of RwCells.

struct PeerReaderCell<T>(UnsafeCell<T>);

unsafe impl<T> Send for PeerReaderCell<T> {}
unsafe impl<T> Sync for PeerReaderCell<T> {}

impl<T> PeerReaderCell<T> {
    fn new(data: T) -> Self {
        return Self(UnsafeCell::new(data));
    }

    fn get(&self) -> *mut T {
        return self.0.get();
    }
}

enum DecodeState {
    Length,
    Payload,
}

struct Peer {
    stream: mio::net::TcpStream,
    token: mio::Token,
    reader: PeerReaderCell<std::io::BufReader<mio::net::TcpStream>>,
    writer: std::io::BufWriter<mio::net::TcpStream>,
    addr: std::net::SocketAddr,
    buffer: PeerReaderCell<Vec<u8>>,
    msg_length: PeerReaderCell<usize>,
    read_length: PeerReaderCell<usize>,
    state: PeerReaderCell<DecodeState>,
}

impl Peer {
    fn new(stream: mio::net::TcpStream, token: mio::Token) -> std::io::Result<Self> {
        let reader_stream = stream.try_clone()?;
        let writer_stream = stream.try_clone()?;
        let addr = stream.peer_addr()?;
        let bufreader = std::io::BufReader::new(reader_stream);
        let bufwriter = std::io::BufWriter::new(writer_stream);
        return Ok(Self {
            stream: stream,
            token: token,
            reader: PeerReaderCell::new(bufreader),
            writer: bufwriter,
            addr: addr,
            buffer: PeerReaderCell::new(vec![0; std::mem::size_of::<u32>()]),
            msg_length: PeerReaderCell::new(std::mem::size_of::<u32>()),
            read_length: PeerReaderCell::new(0),
            state: PeerReaderCell::new(DecodeState::Length),
        });
    }

    fn read(&self) -> std::io::Result<usize> {
        let reader = self.reader.get();
        let buffer = self.buffer.get();
        let msg_length = self.msg_length.get();
        let read_length = self.read_length.get();
        let state = self.state.get();

        unsafe {
            let bytes_read = (*reader).read(&mut (*buffer)[*read_length..*msg_length]);
            match bytes_read {
                Ok(0) => {
                    return Ok(0);
                }
                Ok(size) => {
                    // we got some data, move the cursor
                    *read_length += size;
                    if *read_length == *msg_length {
                        // buffer filled, process the buffer
                        match *state {
                            DecodeState::Length => {
                                let message_length =
                                    BigEndian::read_u32(&(*buffer)[0..std::mem::size_of::<u32>()]);
                                *state = DecodeState::Payload;
                                *read_length = 0;
                                *msg_length = message_length as usize;
                                if (*buffer).capacity() < *msg_length {
                                    (*buffer).resize(*msg_length, 0);
                                }
                            }
                            DecodeState::Payload => {
                                let new_payload: message::Message =
                                    bincode::deserialize(&(*buffer)[0..*msg_length]).unwrap();
                                *state = DecodeState::Length;
                                *read_length = 0;
                                *msg_length = std::mem::size_of::<u32>();
                            }
                        }
                    }
                    return Ok(size);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

pub struct Server {
    peers: RwLock<slab::Slab<Peer>>,
    addr: std::net::SocketAddr,
    poll: mio::Poll,
}

impl Server {
    pub fn start(addr: std::net::SocketAddr) -> std::io::Result<Arc<Self>> {
        let server = Self {
            peers: RwLock::new(slab::Slab::new()),
            addr: addr,
            poll: mio::Poll::new()?,
        };
        let server_ptr = Arc::new(server);
        let server_listening = Arc::clone(&server_ptr);
        thread::spawn(move || {
            server_listening.listen().unwrap_or_else(|e| {
                error!("Error occurred in P2P server: {}", e);
                return;
            });
        });
        return Ok(server_ptr);
    }

    fn register_new_peer(&self, stream: net::TcpStream) -> std::io::Result<()> {
        // get new slot in the connection set
        let mut peers = self.peers.write().unwrap();
        let vacant = peers.vacant_entry();
        let key: usize = vacant.key();
        if key >= MAX_INCOMING_CLIENT {
            // too many connections
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "max peer reached, cannot accept new connections",
            ));
        }
        let new_connection = Peer::new(stream, mio::Token(key))?;
        // register the new connection and insert
        self.poll.register(
            &new_connection.stream,
            new_connection.token,
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;
        vacant.insert(new_connection);
        Ok(())
    }

    fn listen(&self) -> std::io::Result<()> {
        // bind server to passed addr and register to the poll
        let server = net::TcpListener::bind(&self.addr)?;
        const SERVER: mio::Token = mio::Token(std::usize::MAX - 1); // token for server new connection event
        self.poll.register(
            &server,
            SERVER,
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;
        info!(
            "P2P server listening to incoming connections at {}",
            server.local_addr()?
        );

        let mut events = mio::Events::with_capacity(MAX_EVENT);

        loop {
            self.poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        // we have a new connection
                        // we are using edge-triggered events, loop until block
                        loop {
                            // accept the connection
                            match server.accept() {
                                Ok((stream, client_addr)) => {
                                    debug!("New incoming connection from {}", client_addr);
                                    match self.register_new_peer(stream) {
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
                        let peers = self.peers.read().unwrap();
                        let peer = &peers[token_id];
                        // we are using edge-triggered events, loop until block
                        loop {
                            match peer.read() {
                                Ok(0) => {
                                    // EOF, remove it from the connections set
                                    info!("Peer {} dropped connection", peer.addr);
                                    drop(peers);
                                    let mut peers = self.peers.write().unwrap();
                                    peers.remove(token_id);
                                    break;
                                }
                                Ok(_) => {
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
                                        drop(peers);
                                        let mut peers = self.peers.write().unwrap();
                                        // TODO: we did not shutdown the stream. Cool?
                                        peers.remove(token_id);
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
