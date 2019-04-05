use super::message;
use byteorder::{BigEndian, ByteOrder};
use log::{debug, error, info, trace, warn};
use mio::{self, net};
use mio_extras::channel;
use std::io::{Read, Write};
use std::sync::mpsc;

enum DecodeState {
    Length,
    Payload,
}

pub enum ReadResult {
    Continue,
    Message(message::Message),
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
                return Ok(ReadResult::EOF);
            }
            Ok(size) => {
                // we got some data, move the cursor
                self.read_length += size;
                if self.read_length == self.msg_length {
                    // buffer filled, process the buffer
                    match self.state {
                        DecodeState::Length => {
                            let message_length =
                                BigEndian::read_u32(&self.buffer[0..std::mem::size_of::<u32>()]);
                            self.state = DecodeState::Payload;
                            self.read_length = 0;
                            self.msg_length = message_length as usize;
                            if self.buffer.capacity() < self.msg_length {
                                self.buffer.resize(self.msg_length, 0);
                            }
                            return Ok(ReadResult::Continue);
                        }
                        DecodeState::Payload => {
                            let new_payload: message::Message =
                                bincode::deserialize(&self.buffer[0..self.msg_length]).unwrap();
                            self.state = DecodeState::Length;
                            self.read_length = 0;
                            self.msg_length = std::mem::size_of::<u32>();
                            return Ok(ReadResult::Message(new_payload));
                        }
                    }
                } else {
                    return Ok(ReadResult::Continue);
                }
            }
            Err(e) => {
                return Err(e);
            }
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
    pub queue: channel::Receiver<message::Message>,
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
                        self.writer.flush();
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
                        self.msg_buffer = bincode::serialize(&msg).unwrap();
                        self.msg_length = self.msg_buffer.len();
                        BigEndian::write_u32(&mut self.len_buffer, self.msg_length as u32);
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

pub fn new(stream: mio::net::TcpStream) -> std::io::Result<(Context, Handle)> {
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
    };
    let ctx = Context {
        addr: addr,
        stream: stream,
        reader: read_ctx,
        writer: write_ctx,
        handle: handle.clone(),
    };
    return Ok((ctx, handle));
}

pub struct Context {
    pub addr: std::net::SocketAddr,
    pub stream: mio::net::TcpStream,
    pub reader: ReadContext,
    pub writer: WriteContext,
    pub handle: Handle,
}

#[derive(Clone)]
pub struct Handle {
    write_queue: channel::Sender<message::Message>,
}

impl Handle {
    pub fn write(&self, msg: message::Message) {
        self.write_queue.send(msg).unwrap();
        return;
    }
}
