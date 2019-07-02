use prism::visualization::demo;
use std::thread;
use std::time;
use ws::listen;
use websocket::client::ClientBuilder;
use websocket::message::OwnedMessage;

fn main() {
    let server = thread::spawn(move || listen("127.0.0.1:9000", |out| {
        // The handler needs to take ownership of out, so we use move
        move |msg| {
            // Handle messages received on this connection
            println!("Server got message '{}'. ", msg);

            Ok(())
        }
    } ).unwrap());

    server.join().unwrap();
}

