use log::{debug, error, info, trace, warn};
use super::peer::Handle;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Ping(String),
    Pong(String),
}

/// Takes the reference to a message and processes it.
pub fn handle(peer: &Handle, message: &Message) {
    match message {
        Message::Ping(nonce) => {
            info!("Ping message: {}", nonce);
            peer.write(Message::Pong(nonce.to_string()));
        },
        Message::Pong(nonce) => {
            info!("Pong message: {}", nonce);
        },
    }
}
