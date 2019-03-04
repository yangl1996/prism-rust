#[derive(Serialize, Deserialize)]
pub enum Message {
    EchoRequest(String),
    EchoResponse(String),
}
