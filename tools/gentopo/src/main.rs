#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate clap;

use clap::{App, Arg};

#[derive(Serialize, Deserialize)]
struct Topology {
    nodes: Vec<Node>,
    connections: Vec<Connection>,
}

#[derive(Serialize, Deserialize)]
struct Node {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Connection {
    src: String,
    dst: String,
}

fn main() {
    let matches = App::new("gentopo")
        .version("0.1.0")
        .author("Lei Yang <lei@yangl1996.com>")
        .about("Command line tool to generate Prism testbed topologies.")
        .arg(Arg::with_name("topo")
             .short("t")
             .long("topo")
             .help("Topology to use.")
             .takes_value(true)
             .possible_value("clique")
             .required(true)
            )
        .arg(Arg::with_name("num_nodes")
             .short("n")
             .long("nodes")
             .help("Number of nodes in the testbed.")
             .takes_value(true)
             .required(true)
            )
        .arg(Arg::with_name("output_path")
             .short("o")
             .long("output")
             .help("Path of the output file.")
             .takes_value(true)
             .required(true)
            )
        .get_matches();

    let num_nodes: u32 = matches.value_of("num_nodes").unwrap().parse().unwrap();
    let output_path = matches.value_of("output_path").unwrap();
    
    let topo = match matches.value_of("topo").unwrap() {
        "clique" => gen_clique(num_nodes),
        _ => {
            eprintln!("Invalid topology.");
            return;
        },
    };

    let output_path = std::fs::File::create(&output_path).unwrap();
    serde_json::to_writer_pretty(output_path, &topo).unwrap();
    return;
}

fn gen_clique(size: u32) -> Topology {
    let mut nodes: Vec<Node> = Vec::new();
    for node_idx in 0..size {
        let new_node = Node {
            name: format!("node_{}", node_idx),
        };
        nodes.push(new_node);
    }

    let mut conns: Vec<Connection> = Vec::new();
    for dst in 0..size {
        for src in (dst+1)..size {
            let new_conn = Connection {
                src: format!("node_{}", src),
                dst: format!("node_{}", dst),
            };
            conns.push(new_conn);
        }
    }
    return Topology {
        nodes: nodes,
        connections: conns,
    };
}
