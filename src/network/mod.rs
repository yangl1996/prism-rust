pub mod message;
pub mod server;
pub mod peer;

pub fn start(addr: std::net::SocketAddr) -> std::io::Result<server::Handle> {
    let (msg_sink, msg_source) = std::sync::mpsc::channel();
    let (ctx, server) = server::new(addr, msg_sink)?;
    ctx.start();
    return Ok(server);
}
