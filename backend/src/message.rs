use serde::{Deserialize, Serialize};

pub type Port = u16;

#[derive(Serialize, Deserialize)]
pub enum Message {
    Prime {
        msg_id: u32,
        ttl: u32,
        msg_originator: Port,
        msg_forwarder: Port,
        data: u32,
    },
    Ping {
        msg_id: u32,
        msg_originator: Port,
    },
    Pong {
        msg_id: u32,
        msg_originator: Port,
    },
}
