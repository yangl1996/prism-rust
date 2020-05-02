use super::message;




use smol::Async;

pub fn new(
    stream: &Async<std::net::TcpStream>,
) -> std::io::Result<(piper::Receiver<Vec<u8>>, Handle)> {
    let (write_sender, write_receiver) = piper::chan(10000); // TODO: think about the buffer size here
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
    write_queue: piper::Sender<Vec<u8>>,
}

impl Handle {
    pub fn write(&self, msg: message::Message) {
        // TODO: return result
        let buffer = bincode::serialize(&msg).unwrap();
        futures::executor::block_on(self.write_queue.send(buffer));
    }
}
