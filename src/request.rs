use wasm_bindgen::JsCast;

pub async fn get_store(key: &str) -> Option<String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    opts.mode(web_sys::RequestMode::Cors);

    let url = format!("https://colonq.computer/bullfrog/api/get/{}", key);

    let request = web_sys::Request::new_with_str_and_init(&url, &opts).ok()?;

    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await.ok()?;

    assert!(resp_value.is_instance_of::<web_sys::Response>());
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    let text = wasm_bindgen_futures::JsFuture::from(resp.text().ok()?).await.ok()?;
    text.as_string()
}
