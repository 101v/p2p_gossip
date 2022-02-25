use actix_web::{
    client::Client,
    get,
    rt::{spawn, time},
    App, HttpServer, Responder,
};
use backend::{
    message::{Message, Port},
    state::State,
};
use rand::prelude::SliceRandom;
use std::{
    env,
    fs::read_to_string,
    sync::{Arc, Mutex},
    time::Duration,
};

fn ensure_valid_port_number(port: Port) {
    if port < 1024 {
        panic!("Port number must be >= 1024");
    }
}

fn get_names() -> String {
    read_to_string("names.txt").unwrap()
}

fn choose_random_name() -> String {
    get_names()
        .split_whitespace()
        .collect::<Vec<_>>()
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_string()
}

#[allow(dead_code)]
fn schedule<F>(seconds: u64, state: Arc<Mutex<State>>, f: F)
where
    F: 'static + Fn(&mut State),
{
    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(seconds));
        loop {
            interval.tick().await;
            let mut s = state.lock().unwrap();
            f(&mut s);
        }
    });
}

#[get("/")]
async fn index() -> impl Responder {
    "Blockchain node"
}

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

/// Send point-to-point message to a specific peer.
/// Node specific metadata is automatically added.
/// You'll be using this function to send messages.
async fn send_message_to(peer: Port, message: Message) {
    send(peer, serde_json::to_string(&message).unwrap()).await
}

async fn send(peer: Port, message: String) {
    let client = Client::new();

    client
        .post(format!("http://localhost:{}/receive", peer))
        .content_type("application/json")
        .send_body(message)
        .await
        .unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Must pass in port number");
    }

    let port: Port = args[1].parse().expect("Invalid port number");
    ensure_valid_port_number(port);

    let name = choose_random_name();

    println!("Booting node {} ({})", port, name);

    let mut state = State::new(name, port);

    // If passed in another peer's port, initialise that peer
    if args.len() >= 3 {
        let peer: Port = args[2].parse().expect("Invalid peer port number");
        ensure_valid_port_number(peer);
        state.add_peer(peer);
    }

    let shared_state = Arc::new(Mutex::new(state));

    // Send a ping to each of our peers once every 5 seconds
    schedule(5, shared_state.clone(), send_pings_to_everyone);

    // If a peer han't responded to a ping or sent a ping in the last 10 seconds,
    // evict them
    schedule(2, shared_state.clone(), State::evict_stale_peers);

    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
