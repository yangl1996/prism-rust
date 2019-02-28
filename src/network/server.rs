use log::{debug, error, info, warn};
use mio::{self, net};
use std::io::{Read, Write};

const MAX_INCOMING_CLIENT: usize = 256;
const MAX_EVENT: usize = 1024;

struct Connection {
    stream: mio::net::TcpStream,
    token: mio::Token,
}

pub fn p2p_server(addr: std::net::SocketAddr) -> std::io::Result<()> {
    // start a poll (epoll on linux, kqueue on darwin)
    let poll = mio::Poll::new()?;

    // bind server to passed addr and register to the poll
    let server = net::TcpListener::bind(&addr)?;
    const SERVER: mio::Token = mio::Token(std::usize::MAX - 1); // token for server new connection event
    poll.register(
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

    // create a slab to store all connections
    let mut connections = slab::Slab::new();

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    // we have a new connection
                    // we are using edge-triggered events, loop until block
                    loop {
                        // accept the connection
                        match server.accept() {
                            Ok((stream, client_addr)) => {
                                // get new slot in the connection set
                                let vacant = connections.vacant_entry();
                                let key: usize = vacant.key();
                                if key >= MAX_INCOMING_CLIENT {
                                    // too many connections
                                    warn!(
                                        "Incoming client max reached, cannot accept {}",
                                        client_addr
                                    );
                                    continue;
                                }
                                let new_connection = Connection {
                                    stream: stream,
                                    token: mio::Token(key),
                                };
                                // register the new connection and insert
                                poll.register(
                                    &new_connection.stream,
                                    new_connection.token,
                                    mio::Ready::readable(),
                                    mio::PollOpt::edge(),
                                )?;
                                vacant.insert(new_connection);
                                info!("Accepted incoming peer from {}", client_addr);
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
                    let connection = &mut connections[token_id];
                    // we are using edge-triggered events, loop until block
                    loop {
                        let mut buf = [0 as u8; 50];
                        match connection.stream.read(&mut buf) {
                            Ok(0) => {
                                // EOF, remove it from the connections set
                                info!("Peer {} dropped connection", connection.stream.peer_addr()?);
                                connections.remove(token_id);
                                break;
                            }
                            Ok(size) => {
                                // we got some data
                                connection.stream.write(&buf[0..size])?;
                            }
                            Err(e) => {
                                if e.kind() == std::io::ErrorKind::WouldBlock {
                                    // socket is not ready anymore, stop reading
                                    break;
                                } else {
                                    warn!(
                                        "Error reading peer {}, disconnecting: {}",
                                        connection.stream.peer_addr()?,
                                        e
                                    );
                                    connection.stream.shutdown(std::net::Shutdown::Both)?;
                                    connections.remove(token_id);
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
