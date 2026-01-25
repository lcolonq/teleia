use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use crate::net::ws::Message;
use crate::{Erm, WrapErr};

const KEY: usize = 42;

pub struct Client {
    channels: Arc<Mutex<Option<(Sender<Message>, Receiver<Message>)>>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(None)),
        }
    }
    pub fn connect(&mut self, url: &str) {
        let (mut socket, _resp) = if let Ok(v) = tungstenite::connect(url) { v } else {
            log::warn!("failed to connect to websocket: {}", url);
            return;
        };
        let poller = match polling::Poller::new() {
            Ok(v) => v,
            Err(e) => {
                log::warn!("failed to create poller: {}", e);
                return;
            }
        };
        match socket.get_ref() {
            tungstenite::stream::MaybeTlsStream::Plain(s) => unsafe {
                if let Err(e) = poller.add(s, polling::Event::readable(KEY)) {
                    log::warn!("failed to add event to poll: {}", e);
                    let _ = socket.close(None);
                    return;
                };
            },
            tungstenite::stream::MaybeTlsStream::NativeTls(s) => unsafe {
                if let Err(e) = poller.add(s.get_ref(), polling::Event::readable(KEY)) {
                    log::warn!("failed to add event to poll: {}", e);
                    let _ = socket.close(None);
                    return;
                };
            },
            _ => {
                log::warn!("unknown socket type; cannot poll!");
                let _ = socket.close(None);
                return;
            },
        }
        let (outgoing_sender, outgoing_receiver) = channel();
        let (incoming_sender, incoming_receiver) = channel();
        *self.channels.lock().unwrap() = Some((outgoing_sender, incoming_receiver));
        let channels_ref = self.channels.clone();
        let mut events = polling::Events::new();
        spawn(move || loop {
            let res: Erm<()> = try {
                // send all outgoing messages
                while let Ok(msg) = outgoing_receiver.try_recv() {
                    match msg {
                        Message::Text(txt) => socket.send(tungstenite::Message::Text(txt))
                            .wrap_err("failed to send websocket text")?,
                        Message::Binary(bytes) => socket.send(tungstenite::Message::Binary(bytes))
                            .wrap_err("failed to send websocket bytes")?,
                    }
                }
                // wait until we've received data
                events.clear();
                poller.wait(&mut events, None).wrap_err("failed to wait on websocket poller")?;
                for ev in events.iter() {
                    if ev.key == KEY {
                        if ev.readable { match socket.read().wrap_err("failed to recv on websocket")? {
                            tungstenite::Message::Text(txt) => incoming_sender.send(Message::Text(txt))
                                .wrap_err("failed to send incoming websocket message on channel")?,
                            tungstenite::Message::Binary(bytes) => incoming_sender.send(Message::Binary(bytes))
                                .wrap_err("failed to send incoming websocket message on channel")?,
                            tungstenite::Message::Ping(bs) => socket.send(tungstenite::Message::Pong(bs))
                                .wrap_err("failed to send websocket pong")?,
                            m => log::warn!("unhandled incoming websocket message: {}", m),
                        }}
                        match socket.get_ref() {
                            tungstenite::stream::MaybeTlsStream::Plain(s) =>
                                    poller.modify(s, polling::Event::readable(KEY))
                                        .wrap_err("failed to modify event to poll")?,
                            tungstenite::stream::MaybeTlsStream::NativeTls(s) =>
                                    poller.modify(s.get_ref(), polling::Event::readable(KEY))
                                        .wrap_err("failed to modify event to poll")?,
                            _ => log::warn!("unknown socket type; cannot modify polling!"),
                        }
                    }
                }
            };
            if let Err(e) = res {
                log::warn!("error in websocket thread: {}", e);
                *channels_ref.lock().unwrap() = None;
                return;
            }
        });
    }
    pub fn is_connected(&self) -> bool {
        self.channels.lock().unwrap().is_some()
    }
    pub fn poll(&mut self) -> Option<Message> {
        if let Some((_, incoming)) = &*self.channels.lock().unwrap() {
            incoming.try_recv().ok()
        } else { None }
    }
    pub fn send(&self, msg: Message) {
        if let Some((outgoing, _)) = &mut *self.channels.lock().unwrap() {
            if let Err(e) = outgoing.send(msg) {
                log::warn!("failed to send websocket message on channel: {}", e);
            }
        } else {
            log::warn!("tried to send message, but websocket is not connected");
        }
    }
}
