use std::{collections::VecDeque, sync::{Arc, Mutex}};

use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub enum Message {
    Binary(Vec<u8>),
    Text(String),
}

impl Message {
    pub fn from_messageevent(e: web_sys::MessageEvent) -> Self {
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            let array = js_sys::Uint8Array::new(&abuf).to_vec();
            Message::Binary(array)
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            Message::Text(txt.into())
        } else {
            panic!("received weird websocked message: {:?}", e);
        }
    }
}

pub struct Client {
    pub ws: Option<web_sys::WebSocket>,
    pub messages: Arc<Mutex<VecDeque<Message>>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            ws: None,
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn connect(&mut self, url: &str) {
        let ws = web_sys::WebSocket::new(url).expect("failed to open websocket");
        let messages_ref = self.messages.clone();
        let cb: Closure<dyn Fn(web_sys::MessageEvent)> = Closure::new(move |e: web_sys::MessageEvent| {
            let msg = Message::from_messageevent(e);
            log::info!("incoming: {:?}", msg);
            messages_ref.lock().unwrap().push_back(msg);
        });
        ws.set_onmessage(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
        self.ws = Some(ws);
    }
    pub fn poll(&mut self) -> Option<Message> {
        self.messages.lock().unwrap().pop_front()
    }
    pub fn send(&self, msg: Message) {
        if let Some(ws) = &self.ws {
            match msg {
                Message::Text(txt) => {
                    if let Err(e) = ws.send_with_str(&txt) {
                        log::warn!("failed to send string: {:?}", e);
                    }
                },
                Message::Binary(bytes) => {
                    if let Err(e) = ws.send_with_u8_array(&bytes) {
                        log::warn!("failed to send bytes: {:?}", e);
                    }
                },
            }
        }
    }
}
