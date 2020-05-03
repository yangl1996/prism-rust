use super::message;
use futures::{channel::mpsc, sink::SinkExt};
use smol::Async;

pub fn new(
    stream: &Async<std::net::TcpStream>,
) -> std::io::Result<(mpsc::UnboundedReceiver<Vec<u8>>, Handle)> {
    let (write_sender, write_receiver) = mpsc::unbounded(); // TODO: think about the buffer size here
    let addr = stream.get_ref().peer_addr()?;
    let handle = Handle {
        write_queue: write_sender,
        addr,
    };
    Ok((write_receiver, handle))
}

#[derive(Copy, Clone)]
pub enum Direction {
    Incoming,
    Outgoing,
}

#[derive(Clone)]
pub struct Handle {
    addr: std::net::SocketAddr,
    write_queue: mpsc::UnboundedSender<Vec<u8>>,
}

impl Handle {
    pub fn write(&mut self, msg: message::Message) {
        // TODO: return result
        let buffer = bincode::serialize(&msg).unwrap();
        futures::executor::block_on(self.write_queue.send(buffer));
    }
}
