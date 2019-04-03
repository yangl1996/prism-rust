use log::{debug, error, info, trace, warn};
use super::peer::Handle;
use std::sync::{Arc, Mutex};
use crate::blockdb::BlockDatabase;
use crate::blockchain::BlockChain;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Ping(String),
    Pong(String),
}

