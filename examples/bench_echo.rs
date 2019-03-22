use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Instant;

const MSG_SIZE: usize = 1024;
const REPEAT_TIME: usize = 100000;

fn main() {
    stderrlog::new().verbosity(0).init().unwrap();
    let server1_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9998);
    let (ctx, server1) = prism::network::server::new(server1_addr).unwrap();
    ctx.start();

    let server2_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999);
    let (ctx, server2) = prism::network::server::new(server2_addr).unwrap();
    ctx.start();

    let peer = server2.connect(server1_addr).unwrap();

    let message = prism::network::message::Message::Ping("a".repeat(MSG_SIZE));

    let start = Instant::now();
    for _ in 0..REPEAT_TIME {
        let message = prism::network::message::Message::Ping("a".repeat(MSG_SIZE));
        peer.write(message);
    }
    let end = Instant::now();

    let time = end.duration_since(start).as_micros() as f64;
    let throughput = MSG_SIZE as f64 * REPEAT_TIME as f64 * 1000000.0 / time / 1024.0 / 1024.0;
    println!("Message size: {} KB", MSG_SIZE / 1024);
    println!(
        "Throughput: {:.3} MB/s, {:.2} messages/s",
        throughput,
        REPEAT_TIME as f64 * 1000000.0 / time
    );
}
