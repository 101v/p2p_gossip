mod mersenne_prime;
pub mod message;
pub mod state;

// use message::Port;
// use state::State;

// fn handle_prime_message(state: &mut State, msg_originator: Port, _ttl: u32, data: u32) {
//     if state.is_biggest_prime(data) {
//         state.capture_bigget_prime(msg_originator, data);
//         update_last_heard_from(state, msg_originator);
//     }

//     // if ttl > 0 {
//     //     for peer in state.peers.keys() {
//     //         send_message_to(
//     //             peer,
//     //             Message::Prime {
//     //                 ttl: ttl - 1,
//     //                 msg_originator,
//     //                 data,
//     //             },
//     //         )
//     //     }
//     // }
// }

// /// Helper method to log when we last heard from a peer. We have to keep updating when we last heard
// /// from each peer, otherwise stale peers will churn out of our peer list after 10 seconds.
// fn update_last_heard_from(state: &mut State, peer: Port) {
//     state.add_peer(peer);
// }
