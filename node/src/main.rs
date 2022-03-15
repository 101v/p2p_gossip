use backend::state::State;
use crossbeam::channel;
use std::env;
use std::sync::{Arc, RwLock};
use warp::Filter;

mod arguments;
mod filters;
mod handlers;
mod logger;
mod scheduled_actions;

use crate::arguments::parse_name_and_number;
use crate::scheduled_actions::{handler, schedule, Actions};

type ReadState = Arc<RwLock<State>>;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let (port, name, peer) = parse_name_and_number(args);

    println!("Booting node {} ({})", port, name);

    let mut state = State::new(name, port);

    // If passed in another peer's port, initialise that peer
    if let Some(p) = peer {
        state.add_peer(p);
    }

    let shared_state = Arc::new(RwLock::new(state));

    let (tx, rx) = channel::unbounded();

    println!("Starting handler");
    tokio::spawn(handler(rx, shared_state.clone()));

    // Send a ping to each of our peers once every 5 seconds
    println!("Starting Ping scheduler");
    schedule(5, Actions::PingEveryone, tx.clone());

    // If a peer han't responded to a ping or sent a ping in the last 10 seconds,
    // evict them
    println!("Starting evication scheduler");
    schedule(2, Actions::EvictStalePeers, tx.clone());

    // Generare and gossip out a Mersenne prime every 10 seconds
    println!("Starting gossip scheduler");
    schedule(10, Actions::GossipMersennePrime, tx.clone());

    let sender = tx.clone();

    println!("Starting web server");
    let api = filters::get(shared_state.clone()).or(filters::receive(sender));

    warp::serve(api).run(([127, 0, 0, 1], port)).await;
}
