use prism::visualization::demo;
use std::thread;
use std::time;
use ws::listen;
use websocket::client::ClientBuilder;
use websocket::message::OwnedMessage;

fn main() {
    let server = thread::spawn(move || listen("127.0.0.1:2012", |out| {
        // The handler needs to take ownership of out, so we use move
        move |msg| {
            // Handle messages received on this connection
            println!("Server got message '{}'. ", msg);

            Ok(())
        }
    } ).unwrap());
    thread::sleep(time::Duration::from_millis(550));

    /*
    let client = ClientBuilder::new("ws://127.0.0.1:2012")
		.unwrap()
		.add_protocol("rust-websocket")
		.connect_insecure()
		.unwrap();

    println!("Successfully connected");

    let (mut receiver, mut sender) = client.split().unwrap();
    sender.send_message(&OwnedMessage::Text("s".to_string())).unwrap();
    */

    let mut s = demo::Server::new("ws://127.0.0.1:2012");
    s.test("1");
    s.test("2");
    s.test("3");
    s.test("4");

    server.join().unwrap();
    println!("Yo");
}

