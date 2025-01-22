#[cfg(target_arch = "wasm32")]
pub fn main() {}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() {
    teleia::run("teleia test", 240, 160, teleia::TestGame::new).await;
}
