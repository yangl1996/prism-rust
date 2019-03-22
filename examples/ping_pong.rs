use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Instant;
use std::{thread, time};

fn main() {
    stderrlog::new().verbosity(2).init().unwrap();
    let server1_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9998);
    let (ctx, server1) = prism::network::server::new(server1_addr).unwrap();
    ctx.start();

    let server2_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999);
    let (ctx, server2) = prism::network::server::new(server2_addr).unwrap();
    ctx.start();

    thread::sleep(time::Duration::new(1, 0));

    let peer = server2.connect(server1_addr).unwrap();


    let message = prism::network::message::Message::Ping("hello from server 2".to_string());
    peer.write(message);
    
    thread::sleep(time::Duration::new(1, 0));
    println!("Finished.")
}
