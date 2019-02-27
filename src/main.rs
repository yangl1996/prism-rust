#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate clap;

use log::{debug, info, warn, error};

mod hash;
mod merkle;
//mod network;

fn main() {
    let matches = clap_app!(Prism =>
    (version: "0.1")
    (about: "Prism blockchain full client")
    (@arg verbose: -v ... "Increases the verbosity of logging")
    )
    .get_matches();

    let verbosity = matches.occurrences_of("verbose") as usize;

    stderrlog::new().verbosity(verbosity).init().unwrap();

    info!("Server started.");
//    network::server::listener();
}
