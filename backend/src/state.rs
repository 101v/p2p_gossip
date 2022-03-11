use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

use crate::mersenne_prime;
use crate::message::Port;

#[derive(Serialize)]
pub struct State {
    name: String,
    port: Port,
    peers: HashMap<Port, SystemTime>,
    biggest_prime: u32,
    biggest_prime_sender: Port,
    msg_id: u32,
    awake: bool,
    received_messages: HashSet<(Port, u32)>,
}

impl State {
    pub fn new(name: String, port: Port) -> Self {
        State {
            name,
            port,
            peers: HashMap::new(),
            biggest_prime: 2,
            biggest_prime_sender: port,
            msg_id: 0,
            awake: true,
            received_messages: HashSet::new(),
        }
    }

    pub fn add_peer(&mut self, peer: Port) {
        self.peers.insert(peer, SystemTime::now());
    }

    pub fn update_last_heard_from(&mut self, peer: Port) {
        self.add_peer(peer);
    }

    /// Function that evicts any peers who we haven't heard from in the last 10 seconds.
    /// Runs every second.
    pub fn evict_stale_peers(&mut self) {
        self.peers.retain(|_, v| !State::is_stale(v));
    }

    fn is_stale(last_heard: &SystemTime) -> bool {
        if let Ok(duration) = SystemTime::now().duration_since(*last_heard) {
            return duration > Duration::from_secs(10);
        }
        return false;
    }

    pub fn is_biggest_prime(&self, prime: u32) -> bool {
        self.biggest_prime < prime
    }

    pub fn capture_bigget_prime(&mut self, prime_sender: Port, prime: u32) {
        self.biggest_prime = prime;
        self.biggest_prime_sender = prime_sender;
    }

    pub fn get_peers(&self) -> Vec<Port> {
        self.peers.keys().map(|k| *k).collect()
    }

    pub fn get_next_msg_id(&mut self) -> u32 {
        self.msg_id += 1;
        self.msg_id
    }

    pub fn my_port(&self) -> Port {
        self.port
    }

    pub fn generate_next_mersenne_prime(&mut self) -> u32 {
        let next_prime = mersenne_prime::find_next_mersenne_prime(self.biggest_prime);
        self.biggest_prime = next_prime;
        self.biggest_prime_sender = self.port;

        self.biggest_prime
    }

    pub fn has_seen_message(&self, msg_originator: Port, msg_id: u32) -> bool {
        self.received_messages.contains(&(msg_originator, msg_id))
    }

    pub fn record_received_message(&mut self, msg_originator: Port, msg_id: u32) {
        self.received_messages.insert((msg_originator, msg_id));
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Sub,
        time::{Duration, SystemTime},
    };

    use super::State;

    #[test]
    fn peer_should_not_be_repeated() {
        let mut state = State::new(String::from("node1"), 5000);
        state.add_peer(5001);
        state.add_peer(5002);
        state.add_peer(5001);

        assert_eq!(state.peers.len(), 2);
    }

    #[test]
    fn stale_peers_should_be_evicted() {
        let mut state = State::new(String::from("node1"), 5000);
        state.add_peer(5001);
        state.add_peer(5002);
        state
            .peers
            .insert(5003, SystemTime::now().sub(Duration::from_secs(11)));

        assert_eq!(state.peers.len(), 3);
        state.evict_stale_peers();
        assert_eq!(state.peers.len(), 2);
    }
}
