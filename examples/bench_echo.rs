use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use byteorder::{BigEndian, ByteOrder};
use std::io::Write;
use std::time::{Duration, Instant};

const MSG_SIZE: usize = 1024 ;
const REPEAT_TIME: usize = 100000;

fn main() {
    let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999);
    let mut server = prism::network::server::Server::new(listen_addr).unwrap();
    thread::spawn(move|| {
        server.listen().unwrap();
    });
    
    let message = prism::network::message::Message::EchoRequest("a".repeat(MSG_SIZE));
    let encoded: Vec<u8> = bincode::serialize(&message).unwrap();
    let length: u32 = encoded.len() as u32;
    let mut length_encoded = [0; 4];
    BigEndian::write_u32(&mut length_encoded, length);

    let socket = std::net::TcpStream::connect("127.0.0.1:9999").unwrap();
    let mut writer = std::io::BufWriter::new(socket);

    let start = Instant::now();
    for i in 0..REPEAT_TIME {
        writer.write(&length_encoded).unwrap();
        writer.write(&encoded).unwrap();
    }
    writer.flush();
    let end = Instant::now();
    let time = end.duration_since(start).as_micros() as f64;
    let throughput = MSG_SIZE as f64 * REPEAT_TIME as f64 * 1000000.0 / time / 1024.0 / 1024.0;
    println!("Throughput: {:.3} MB/s", throughput);
}
