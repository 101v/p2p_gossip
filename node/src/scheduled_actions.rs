use backend::message::{Message, Port};
use backend::state::State;
use crossbeam::channel::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

fn prepare_ping_messages(state: Arc<RwLock<State>>) -> HashMap<Port, Message> {
    let mut state = state.write().unwrap();
    let mut messages_to_send = HashMap::new();

    for peer in state.get_peers() {
        messages_to_send.insert(
            peer,
            Message::Ping {
                msg_id: state.get_next_msg_id(),
                msg_originator: state.my_port(),
            },
        );
    }

    messages_to_send
}

async fn send_pings_to_everyone(state: Arc<RwLock<State>>) {
    let messages_to_send = prepare_ping_messages(state);

    for (peer, message) in messages_to_send {
        send_message_to(peer, message).await;
    }
}

fn evict_stale_peers(state: Arc<RwLock<State>>) {
    let mut state = state.write().unwrap();
    state.evict_stale_peers();
}

fn generate_next_prime_message(state: Arc<RwLock<State>>) -> (Vec<Port>, Message) {
    let mut state = state.write().unwrap();
    let msg_originator = state.my_port();
    let data = state.generate_next_mersenne_prime();
    let msg_id = state.get_next_msg_id();

    (
        state.get_peers(),
        Message::Prime {
            msg_id,
            ttl: 2,
            msg_originator,
            msg_forwarder: msg_originator,
            data,
        },
    )
}

async fn generate_and_gosspit_next_mersenne_prime(state: Arc<RwLock<State>>) {
    let (peers, message) = generate_next_prime_message(state);

    for peer in peers {
        send_message_to(peer, message).await;
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

#[derive(Clone, Copy)]
pub enum Actions {
    PingEveryone,
    EvictStalePeers,
    GossipMersennePrime,
    Respond(Message),
}

fn checked_and_recorded(
    msg_id: u32,
    msg_originator: Port,
    msg_forwarder: Port,
    state: Arc<RwLock<State>>,
) -> bool {
    let mut state = state.write().unwrap();

    if state.has_seen_message(msg_originator, msg_id) || msg_originator == state.my_port() {
        return false;
    }
    state.record_received_message(msg_originator, msg_id);
    state.update_last_heard_from(msg_forwarder);
    true
}

fn prepare_pong_message(state: Arc<RwLock<State>>) -> Message {
    let mut state = state.write().unwrap();
    Message::Pong {
        msg_originator: state.my_port(),
        msg_id: state.get_next_msg_id(),
    }
}

fn prepare_prime_message(
    msg_id: u32,
    ttl: u32,
    msg_originator: Port,
    data: u32,
    state: Arc<RwLock<State>>,
) -> (Message, Vec<Port>) {
    let state = state.write().unwrap();
    (
        Message::Prime {
            msg_id,
            msg_forwarder: state.my_port(),
            ttl: ttl - 1,
            msg_originator,
            data,
        },
        state.get_peers(),
    )
}

async fn respond(message: Message, state: Arc<RwLock<State>>) {
    match message {
        Message::Ping {
            msg_id,
            msg_originator,
        } => {
            if !checked_and_recorded(msg_id, msg_originator, msg_originator, state.clone()) {
                return;
            }

            let pong_message = prepare_pong_message(state);
            send_message_to(msg_originator, pong_message).await;
        }

        Message::Pong {
            msg_id,
            msg_originator,
        } => {
            checked_and_recorded(msg_id, msg_originator, msg_originator, state);
        }

        Message::Prime {
            msg_id,
            ttl,
            msg_originator,
            msg_forwarder,
            data,
        } => {
            if !checked_and_recorded(msg_id, msg_originator, msg_forwarder, state.clone()) {
                return;
            }

            if ttl > 0 {
                let (message_to_forward, peers) =
                    prepare_prime_message(msg_id, ttl, msg_originator, data, state);
                for peer in peers {
                    send_message_to(peer, message_to_forward).await;
                }
            }
        }
    }
}

pub(crate) async fn handler(rx: Receiver<Actions>, state: Arc<RwLock<State>>) {
    println!("Handler function called");
    while let Ok(action) = rx.recv() {
        let state = state.clone();
        match action {
            Actions::PingEveryone => send_pings_to_everyone(state).await,
            Actions::EvictStalePeers => evict_stale_peers(state),
            Actions::GossipMersennePrime => generate_and_gosspit_next_mersenne_prime(state).await,
            Actions::Respond(message) => respond(message, state).await,
        }
    }
}

pub(crate) fn schedule(seconds: u64, action: Actions, tx: Sender<Actions>) {
    println!("Scheduler function called");

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(seconds));
        // println!("Scheduled {:?}", action);
        loop {
            interval.tick().await;
            tx.send(action).unwrap();
        }
    });
}
