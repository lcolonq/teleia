pub mod client;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[derive(Debug, Clone)]
pub enum Message {
    Binary(Vec<u8>),
    Text(String),
}
