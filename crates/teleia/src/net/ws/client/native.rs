use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use crate::net::ws::Message;
use crate::{utils, Erm, WrapErr};

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
            Err(e) => { log::warn!("failed to create poller: {}", e); return; }
        };
        let res: Erm<()> = try { match socket.get_ref() {
            tungstenite::stream::MaybeTlsStream::Plain(s) => unsafe {
                s.set_nonblocking(true).wrap_err("failed to set socket nonblocking")?;
                poller.add(s, polling::Event::readable(KEY)).wrap_err("failed to add event to poll")?;
            },
            tungstenite::stream::MaybeTlsStream::NativeTls(s) => unsafe {
                s.get_ref().set_nonblocking(true).wrap_err("failed to set socket nonblocking")?;
                poller.add(s.get_ref(), polling::Event::readable(KEY)).wrap_err("failed to add event to poll")?;
            },
            _ => utils::erm_msg("unknown socket type; cannot poll!")?,
        }};
        if let Err(e) = res {
            log::warn!("failed to add event to poll: {}", e);
            let _ = socket.close(None);
            return;
        };
        let (outgoing_sender, outgoing_receiver) = channel();
        let (incoming_sender, incoming_receiver) = channel();
        *self.channels.lock().unwrap() = Some((outgoing_sender, incoming_receiver));
        let send_channels_ref = self.channels.clone();
        let recv_channels_ref = self.channels.clone();
        let send_socket_ref = Arc::new(Mutex::new(socket));
        let recv_socket_ref = send_socket_ref.clone();
        let mut events = polling::Events::new();
        spawn(move || loop {
            // write all outgoing messages
            let res: Erm<()> = try {
                while let Ok(msg) = outgoing_receiver.recv() {
                    match msg {
                        Message::Text(txt) => send_socket_ref.lock().unwrap()
                            .send(tungstenite::Message::Text(txt))
                            .wrap_err("failed to send websocket text")?,
                        Message::Binary(bytes) => send_socket_ref.lock().unwrap()
                            .send(tungstenite::Message::Binary(bytes))
                            .wrap_err("failed to send websocket bytes")?,
                    }
                }
            };
            if let Err(e) = res {
                log::warn!("error in websocket send thread: {}", e);
                *send_channels_ref.lock().unwrap() = None;
                return;
            }
        });
        spawn(move || loop {
            // wait for and receive all incoming messages
            let res: Erm<()> = try {
                events.clear();
                poller.wait(&mut events, None).wrap_err("failed to wait on websocket poller")?;
                for ev in events.iter() {
                    if ev.key == KEY {
                        if ev.readable { match recv_socket_ref.lock().unwrap().read().wrap_err("failed to recv on websocket")? {
                            tungstenite::Message::Text(txt) => incoming_sender.send(Message::Text(txt))
                                .wrap_err("failed to send incoming websocket message on channel")?,
                            tungstenite::Message::Binary(bytes) => incoming_sender.send(Message::Binary(bytes))
                                .wrap_err("failed to send incoming websocket message on channel")?,
                            tungstenite::Message::Ping(bs) => recv_socket_ref.lock().unwrap()
                                .send(tungstenite::Message::Pong(bs))
                                .wrap_err("failed to send websocket pong")?,
                            m => log::warn!("unhandled incoming websocket message: {}", m),
                        }}
                        match recv_socket_ref.lock().unwrap().get_ref() {
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
                log::warn!("error in websocket recv thread: {}", e);
                *recv_channels_ref.lock().unwrap() = None;
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
