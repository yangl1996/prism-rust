use super::message;
use byteorder::{BigEndian, ByteOrder};
use log::{debug, error, info, trace, warn};
use mio::{self, net};
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

pub struct Context {
    stream: mio::net::TcpStream,
    token: mio::Token,
    reader: std::io::BufReader<mio::net::TcpStream>,
    writer: std::io::BufWriter<mio::net::TcpStream>,
    pub addr: std::net::SocketAddr,
    buffer: Vec<u8>,
    msg_length: usize,
    read_length: usize,
    state: DecodeState,
}

impl Context {
    pub fn new(stream: mio::net::TcpStream, token: mio::Token) -> std::io::Result<Self> {
        let reader_stream = stream.try_clone()?;
        let writer_stream = stream.try_clone()?;
        let addr = stream.peer_addr()?;
        let bufreader = std::io::BufReader::new(reader_stream);
        let bufwriter = std::io::BufWriter::new(writer_stream);
        return Ok(Self {
            stream: stream,
            token: token,
            reader: bufreader,
            writer: bufwriter,
            addr: addr,
            buffer: vec![0; std::mem::size_of::<u32>()],
            msg_length: std::mem::size_of::<u32>(),
            read_length: 0,
            state: DecodeState::Length,
        });
    }

    /*
    pub fn write(&self, message: &message::Message) -> std::io::Result<()> {
        let encoded: Vec<u8> = bincode::serialize(message).unwrap();
        let msg_length: u32 = encoded.len() as u32;
        let mut encoded_length = [0; 4];
        BigEndian::write_u32(&mut encoded_length, msg_length);
        let mut writer = self.writer.lock().unwrap();

        let mut cursor = 0;
        loop {
            match (*writer).write(&encoded_length[cursor..4]) {
                Ok(size) => cursor += size,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // this op would have blocked. try again
                        continue;
                    }
                    else {
                        return Err(e);
                    }
                }
            }

            if cursor == 4 {
                break;
            }
        }

        let mut cursor = 0;
        loop {
            match (*writer).write(&encoded[cursor..msg_length as usize]) {
                Ok(size) => cursor += size,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // this op would have blocked. try again
                        continue;
                    }
                    else {
                        return Err(e);
                    }
                }
            }

            if cursor == msg_length as usize {
                break;
            }
        }

        return Ok(());
    }

    pub fn flush(&self) -> std::io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        (*writer).flush()?;
        return Ok(());
    }
    */

    pub fn read(&mut self) -> std::io::Result<ReadResult> {
        let bytes_read = self.reader.read(&mut self.buffer[self.read_length..self.msg_length]);
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

