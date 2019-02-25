#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate clap;

mod hash;
mod merkle;
mod network;

fn main() {
    let matches = clap_app!(Prism =>
                            (version: "0.1")
                            (about: "Prism blockchain full client")
                            )
        .get_matches();
    println!("Starting server");
    network::server::serve();
}
