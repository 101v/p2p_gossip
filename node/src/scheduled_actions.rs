use backend::message::{Message, Port};
use backend::state::State;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::Duration;

fn send_pings_to_everyone(state: &mut State) {
    for peer in state.get_peers() {
        futures::executor::block_on(send_message_to(
            peer,
            Message::Ping {
                msg_id: state.get_next_msg_id(),
                msg_originator: state.my_port(),
            },
        ));
    }
}

fn generate_and_gosspit_next_mersenne_prime(state: &mut State) {
    let my_port = state.my_port();
    let next_prime = state.generate_next_mersenne_prime();

    for peer in state.get_peers() {
        futures::executor::block_on(send_message_to(
            peer,
            Message::Prime {
                msg_id: state.get_next_msg_id(),
                ttl: 2,
                msg_originator: my_port,
                msg_forwarder: my_port,
                data: next_prime,
            },
        ));
    }
}

/// Send point-to-point message to a specific peer.
/// Node specific metadata is automatically added.
/// You'll be using this function to send messages.
async fn send_message_to(peer: Port, message: Message) {
    send(peer, serde_json::to_string(&message).unwrap()).await
}

async fn send(peer: Port, message: String) {
    let client = reqwest::Client::new();

    client
        .post(format!("http://localhost:{}/receive", peer))
        .body(message)
        .send()
        .await
        .unwrap();
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Actions {
    PingEveryone,
    EvictStalePeers,
    GossipMersennePrime,
}

pub(crate) async fn handler(rx: Receiver<Actions>, state: Arc<RwLock<State>>) {
    println!("Handler function called");
    while let Ok(action) = rx.recv() {
        println!("{:#?}", action);
        let mut s = state.write().unwrap();
        match action {
            Actions::PingEveryone => send_pings_to_everyone(&mut *s),
            Actions::EvictStalePeers => State::evict_stale_peers(&mut *s),
            Actions::GossipMersennePrime => generate_and_gosspit_next_mersenne_prime(&mut *s),
        }
    }
}

pub(crate) fn schedule(seconds: u64, action: Actions, tx: Sender<Actions>) {
    println!("Scheduler function called");

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(seconds));
        println!("Scheduled {:?}", action);
        loop {
            interval.tick().await;
            tx.send(action).unwrap();
        }
    });
}
