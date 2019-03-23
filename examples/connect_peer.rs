use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time;
use std::thread;

fn main() {
    let port = 8000;
    let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let mut servers = vec![];

    for i in 0..5 {
        let listen_addr = SocketAddr::new(localhost, port + i);
        let server = prism::network::start(listen_addr).unwrap();
        servers.push(server);
        println!("Server {} started", i);
    }


    thread::sleep(time::Duration::new(1, 0));
    for i in 0..5 {
        for j in i+1..5 {
            servers[i].connect(SocketAddr::new(localhost, port + j as u16)).unwrap();
            println!("Server {} connected to {}", i, j);
        }
    }
}
