use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn log_info(msg: i8) {
    log::info!("{:?}", msg);
}

#[wasm_bindgen(module="/src/js/module.js")]
extern "C" {
    fn js_build_interface() -> js_sys::Object;
}

pub struct Module {
    pub wasm: js_sys::WebAssembly::Instance,
}

impl Module {
    pub async fn new(bytes: &[u8]) -> Option<Self> {
        let imp = js_build_interface();
        let o = wasm_bindgen_futures::JsFuture::from(
            js_sys::WebAssembly::instantiate_buffer(bytes, &imp)
        ).await.unwrap();
        let i = js_sys::Reflect::get(&o, &"instance".into()).unwrap();
        if let Ok(wasm) = i.dyn_into::<js_sys::WebAssembly::Instance>() {
            Some(Self {
                wasm,
            })
        } else {
            log::info!("failed 3");
            None
        }
    }
    pub fn call(&self, nm: &str) {
        let exp = self.wasm.exports();
        if let Ok(fo) = js_sys::Reflect::get(&exp, &nm.into()) {
            if let Ok(func) = fo.dyn_into::<js_sys::Function>() {
                let _ = func.call0(&JsValue::undefined());
            } else {
                log::warn!("couldn't cast module function: {}", nm);
            }
        } else {
            log::warn!("couldn't find module function: {}", nm);
        }
    }
}
