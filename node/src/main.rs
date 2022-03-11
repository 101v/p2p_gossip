use backend::state::State;
use crossbeam::channel;
use std::env;
use std::sync::{Arc, RwLock};
use warp::Filter;

mod arguments;
mod scheduled_actions;

use crate::arguments::parse_name_and_number;
use crate::scheduled_actions::{handler, schedule, Actions};

type ReadState = Arc<RwLock<State>>;

mod handlers {
    use crate::{scheduled_actions::Actions, ReadState};
    use backend::message::Message;
    use core::result::Result;
    use crossbeam::channel::Sender;
    use std::convert::Infallible;

    pub async fn get(state: ReadState) -> Result<impl warp::Reply, Infallible> {
        let state = state.read().unwrap();
        Ok(warp::reply::json(&*state))
    }

    pub async fn receive(
        message: Message,
        sender: Sender<Actions>,
    ) -> Result<impl warp::Reply, Infallible> {
        sender.send(Actions::Respond(message)).unwrap();
        Ok("Received")
    }
}

mod filters {
    use super::handlers;
    use crate::{scheduled_actions::Actions, ReadState};
    use backend::message;
    use crossbeam::channel::Sender;
    use std::convert::Infallible;
    use warp::Filter;

    pub fn get(
        state: ReadState,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("state")
            .and(warp::get())
            .and(with_state(state))
            .and_then(handlers::get)
    }

    pub fn receive(
        sender: Sender<Actions>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("receive")
            .and(warp::post())
            .and(json_body())
            .and(with_sender(sender))
            .and_then(handlers::receive)
    }

    fn with_state(
        state: ReadState,
    ) -> impl Filter<Extract = (ReadState,), Error = Infallible> + Clone {
        warp::any().map(move || state.clone())
    }

    fn with_sender(
        sender: Sender<Actions>,
    ) -> impl Filter<Extract = (Sender<Actions>,), Error = Infallible> + Clone {
        warp::any().map(move || sender.clone())
    }

    fn json_body() -> impl Filter<Extract = (message::Message,), Error = warp::Rejection> + Clone {
        warp::body::json()
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
