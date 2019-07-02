use prism::visualization::demo;
use std::thread;
use std::time;
use ws::listen;

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
    let s = demo::Server::new("ws://127.0.0.1:2012").unwrap();
    s.test("A").unwrap();
    println!("Yo");
}

