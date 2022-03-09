use backend::message::Port;
use rand::prelude::SliceRandom;
use std::fs::read_to_string;

fn ensure_valid_port_number(port: Port) {
    if port < 1024 {
        panic!("Port number must be >= 1024");
    }
}

fn choose_random_name() -> String {
    read_to_string("names.txt")
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>()
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_string()
}

pub(crate) fn parse_name_and_number(args: Vec<String>) -> (Port, String, Option<Port>) {
    if args.len() < 2 {
        panic!("Must pass in port number");
    }

    let port: Port = args[1].parse().expect("Invalid port number");
    ensure_valid_port_number(port);

    let mut peer = None;

    if args.len() >= 3 {
        let port: Port = args[2].parse().expect("Invalid peer port number");
        ensure_valid_port_number(port);
        peer = Some(port);
    }

    let name = choose_random_name();
    (port, name, peer)
}
