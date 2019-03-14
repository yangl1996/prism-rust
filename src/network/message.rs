use log::{debug, error, info, trace, warn};
use super::server::Peer;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Ping(String),
    Pong(String),
}

/// Takes the reference to a message and processes it.
pub fn handle_message(peer: &Peer, message: &Message) {
    match message {
        Message::Ping(nonce) => {
            info!("Ping message from {}: {}", peer.addr, nonce);
            peer.write(&Message::Pong(nonce.to_string()));
        },
        Message::Pong(nonce) => {
            info!("Pong message from {}: {}", peer.addr, nonce);
        },
    }
}
