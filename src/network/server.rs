use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_tcp_server;

#[rpc]
pub trait Rpc {
    #[rpc(name = "echo")]
    fn echo(&self, a: String) -> Result<String>;
}

pub struct RpcImpl;
impl Rpc for RpcImpl {
    fn echo(&self, a: String) -> Result<String> {
        Ok(a)
    }
}

pub fn serve() {
    let mut io = jsonrpc_core::IoHandler::new();
    io.extend_with(RpcImpl.to_delegate());

    let server = jsonrpc_tcp_server::ServerBuilder::new(io)
        .start(&"127.0.0.1:3030".parse().unwrap())
        .expect("error starting server");
    server.wait();
}
