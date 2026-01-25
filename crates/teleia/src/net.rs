pub mod ws;

#[cfg(not(target_arch = "wasm32"))]
pub mod fig;
