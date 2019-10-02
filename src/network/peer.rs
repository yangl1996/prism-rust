use super::message;
use log::{trace, warn};
use mio;
use mio_extras::channel;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::sync::mpsc;

enum DecodeState {
    Length,
    Payload,
}

pub enum ReadResult {
    Continue,
    Message(Vec<u8>),
    EOF,
}

pub struct ReadContext {
    reader: std::io::BufReader<mio::net::TcpStream>,
    buffer: Vec<u8>,
    msg_length: usize,
    read_length: usize,
    state: DecodeState,
}

impl ReadContext {
    pub fn read(&mut self) -> std::io::Result<ReadResult> {
        let bytes_read = self
            .reader
            .read(&mut self.buffer[self.read_length..self.msg_length]);
        match bytes_read {
            Ok(0) => {
                trace!("Detected socket EOF");
                Ok(ReadResult::EOF)
            }
            Ok(size) => {
                trace!("Read {} bytes from socket", size);
                // we got some data, move the cursor
                self.read_length += size;
                if self.read_length == self.msg_length {
                    // buffer filled, process the buffer
                    match self.state {
                        DecodeState::Length => {
                            let message_length =
                                u32::from_be_bytes(self.buffer[0..4].try_into().unwrap());
                            self.state = DecodeState::Payload;
                            self.read_length = 0;
                            self.msg_length = message_length as usize;
                            if self.buffer.len() < self.msg_length {
                                self.buffer.resize(self.msg_length, 0);
                            }
                            trace!("Received message length={}", message_length);
                            Ok(ReadResult::Continue)
                        }
                        DecodeState::Payload => {
                            let new_payload: Vec<u8> = self.buffer[0..self.msg_length].to_vec();
                            self.state = DecodeState::Length;
                            self.read_length = 0;
                            self.msg_length = std::mem::size_of::<u32>();
                            trace!("Received full message");
                            Ok(ReadResult::Message(new_payload))
                        }
                    }
                } else {
                    Ok(ReadResult::Continue)
                }
            }
            Err(e) => Err(e),
        }
    }
}

pub enum WriteResult {
    Complete,
    EOF,
    ChanClosed,
}

enum WriteState {
    Length,
    Payload,
}

pub struct WriteContext {
    writer: std::io::BufWriter<mio::net::TcpStream>,
    pub queue: channel::Receiver<Vec<u8>>,
    len_buffer: [u8; std::mem::size_of::<u32>()],
    msg_buffer: Vec<u8>,
    msg_length: usize,
    written_length: usize,
    state: WriteState,
}

impl WriteContext {
    pub fn write(&mut self) -> std::io::Result<WriteResult> {
        loop {
            match self.state {
                WriteState::Length => {
                    if self.written_length == std::mem::size_of::<u32>() {
                        // if the length part has been fully sent
                        self.written_length = 0;
                        self.state = WriteState::Payload;
                        continue;
                    } else {
                        // we are still sending the length part
                        let written = self.writer.write(
                            &self.len_buffer[self.written_length..std::mem::size_of::<u32>()],
                        )?;
                        if written == 0 {
                            return Ok(WriteResult::EOF);
                        }
                        self.written_length += written;
                        continue;
                    }
                }
                WriteState::Payload => {
                    if self.written_length == self.msg_length {
                        // if the previous message has been fully written, try to get the next message
                        // first flush the writer
                        self.writer.flush()?;
                        let msg = match self.queue.try_recv() {
                            Ok(msg) => msg,
                            Err(e) => match e {
                                mpsc::TryRecvError::Empty => return Ok(WriteResult::Complete),
                                mpsc::TryRecvError::Disconnected => {
                                    return Ok(WriteResult::ChanClosed);
                                }
                            },
                        };

                        // encode the message and the length
                        self.msg_buffer = msg;
                        self.msg_length = self.msg_buffer.len();
                        self.len_buffer[..4]
                            .copy_from_slice(&(self.msg_length as u32).to_be_bytes());
                        self.written_length = 0;
                        self.state = WriteState::Length;
                        continue;
                    } else {
                        // we are still sending the payload
                        let written = self
                            .writer
                            .write(&self.msg_buffer[self.written_length..self.msg_length])?;
                        if written == 0 {
                            return Ok(WriteResult::EOF);
                        }
                        self.written_length += written;
                        continue;
                    }
                }
            }
        }
    }
}

pub fn new(
    stream: mio::net::TcpStream,
    direction: Direction,
) -> std::io::Result<(Context, Handle)> {
    let reader_stream = stream.try_clone()?;
    let writer_stream = stream.try_clone()?;
    let addr = stream.peer_addr()?;
    let bufreader = std::io::BufReader::new(reader_stream);
    let read_ctx = ReadContext {
        reader: bufreader,
        buffer: vec![0; std::mem::size_of::<u32>()],
        msg_length: std::mem::size_of::<u32>(),
        read_length: 0,
        state: DecodeState::Length,
    };
    let bufwriter = std::io::BufWriter::new(writer_stream);
    let (write_sender, write_receiver) = channel::channel();
    let write_ctx = WriteContext {
        writer: bufwriter,
        queue: write_receiver,
        len_buffer: [0; std::mem::size_of::<u32>()],
        msg_buffer: Vec::new(),
        msg_length: 0,
        written_length: 0,
        state: WriteState::Payload,
    };
    let handle = Handle {
        write_queue: write_sender,
        addr,
    };
    let ctx = Context {
        addr,
        stream,
        reader: read_ctx,
        writer: write_ctx,
        handle: handle.clone(),
        direction,
    };
    Ok((ctx, handle))
}

#[derive(Copy, Clone)]
pub enum Direction {
    Incoming,
    Outgoing,
}

pub struct Context {
    pub addr: std::net::SocketAddr,
    pub stream: mio::net::TcpStream,
    pub reader: ReadContext,
    pub writer: WriteContext,
    pub handle: Handle,
    pub direction: Direction,
}

#[derive(Clone)]
pub struct Handle {
    addr: std::net::SocketAddr,
    write_queue: channel::Sender<Vec<u8>>,
}

impl Handle {
    pub fn write(&self, msg: message::Message) {
        // TODO: return result
        let buffer = bincode::serialize(&msg).unwrap();
        if self.write_queue.send(buffer).is_err() {
            warn!("Failed to send write request for peer {}, channel detached", self.addr);
        }
    }
}
