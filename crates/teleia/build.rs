fn main() {
    match &*std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
        "x86_64" => if let Ok(p) = std::env::var("LIBCOLONQ_PIT_NATIVE") {
            println!("cargo::rustc-link-search={}", p);
        },
        "wasm32" => if let Ok(p) = std::env::var("LIBCOLONQ_PIT_WASM") {
            println!("cargo::rustc-link-search={}", p);
        },
        _ => eprintln!("warning: building for unknown architecture!"),
    }
    println!("cargo::rustc-link-lib=static=colonq-pit");
}
