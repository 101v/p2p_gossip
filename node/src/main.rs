use backend::state::State;
use std::env;
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};

mod arguments;
mod scheduled_actions;

use crate::arguments::parse_name_and_number;
use crate::scheduled_actions::{handler, schedule, Actions};

type ReadState = Arc<RwLock<State>>;

mod handlers {
    use core::result::Result;
    use std::convert::Infallible;

    use crate::ReadState;

    pub async fn get_state(state: ReadState) -> Result<impl warp::Reply, Infallible> {
        let state = state.read().unwrap();
        Ok(warp::reply::json(&*state))
    }
}

mod filters {
    use super::handlers;
    use crate::ReadState;
    use std::convert::Infallible;
    use warp::Filter;

    pub fn get_state(
        state: ReadState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("state")
            .and(warp::get())
            .and(with_state(state))
            .and_then(handlers::get_state)
    }

    fn with_state(
        state: ReadState,
    ) -> impl Filter<Extract = (ReadState,), Error = Infallible> + Clone {
        warp::any().map(move || state.clone())
    }
}

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

    let (tx, rx) = channel();

    // println!("Starting handler");
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

    // let sender = tx.clone();

    println!("Starting web server");
    let api = filters::get_state(shared_state.clone());
    warp::serve(api).run(([127, 0, 0, 1], port)).await;
}
