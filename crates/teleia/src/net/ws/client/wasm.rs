use std::{collections::VecDeque, sync::{Arc, Mutex}};

use wasm_bindgen::prelude::*;

use crate::net::ws::Message;

impl Message {
    fn from_messageevent(e: web_sys::MessageEvent) -> Self {
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            let array = js_sys::Uint8Array::new(&abuf).to_vec();
            Message::Binary(array)
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            Message::Text(txt.into())
        } else {
            panic!("received weird websocket message: {:?}", e);
        }
    }
}

pub struct Client {
    ws: Arc<Mutex<Option<web_sys::WebSocket>>>,
    messages: Arc<Mutex<VecDeque<Message>>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            ws: Arc::new(Mutex::new(None)),
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn connect(&mut self, url: &str) {
        let ws_ref = self.ws.clone();
        let messages_ref = self.messages.clone();
        let cb: Closure<dyn Fn(web_sys::MessageEvent)> = Closure::new(move |e: web_sys::MessageEvent| {
            let msg = Message::from_messageevent(e);
            log::info!("incoming: {:?}", msg);
            messages_ref.lock().unwrap().push_back(msg);
        });
        let close_cb: Closure<dyn Fn(web_sys::MessageEvent)> = Closure::new(move |_: web_sys::MessageEvent| {
            log::info!("closed!");
            *ws_ref.lock().unwrap() = None;
        });
        let ws = web_sys::WebSocket::new(url).expect("failed to open websocket");
        ws.set_onmessage(Some(cb.as_ref().unchecked_ref()));
        ws.set_onclose(Some(close_cb.as_ref().unchecked_ref()));
        cb.forget();
        close_cb.forget();
        *self.ws.lock().unwrap() = Some(ws);
    }
    pub fn is_connected(&self) -> bool {
        self.ws.lock().unwrap().is_none()
    }
    pub fn poll(&mut self) -> Option<Message> {
        self.messages.lock().unwrap().pop_front()
    }
    pub fn send(&self, msg: Message) {
        if let Some(ws) = &*self.ws.lock().unwrap() {
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
