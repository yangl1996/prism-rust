use prism::visualization::demo;

fn main() {
    demo::Server::new("ws://localhost:2012").unwrap();
    println!("Yo");
}

