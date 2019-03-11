#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    EchoRequest(String),
    EchoResponse(String),
}
