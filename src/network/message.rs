use crate::block::Block;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    Block(Vec<Block>),
}
