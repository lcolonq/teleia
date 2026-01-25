pub mod client;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[derive(Debug)]
pub enum Message {
    Binary(Vec<u8>),
    Text(String),
}
