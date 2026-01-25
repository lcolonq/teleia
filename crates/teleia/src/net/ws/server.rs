use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use crate::net::ws::Message;
use crate::{Erm, WrapErr};

const ACCEPT_KEY: usize = 1;

type ClientId = usize;
type ClientMessage = (ClientId, Message);

pub struct Server {
    clients: Arc<Mutex<HashMap<ClientId, tungstenite::WebSocket<TcpStream>>>>,
    channels: Arc<Mutex<Option<(Sender<ClientMessage>, Receiver<ClientMessage>)>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(None)),
        }
    }
    pub fn start(&mut self, addr: &str) {
        let clients_ref = self.clients.clone();
        let (outgoing_sender, outgoing_receiver) = channel();
        let (incoming_sender, incoming_receiver) = channel();
        *self.channels.lock().unwrap() = Some((outgoing_sender, incoming_receiver));
        let channels_ref = self.channels.clone();
        let listener = TcpListener::bind(addr).expect("failed to bind server socket");
        let poller = polling::Poller::new().expect("failed to create poller for websocket server");
        unsafe {
            poller.add(&listener, polling::Event::readable(ACCEPT_KEY))
                .expect("failed to add poll server socket");
        }
        let mut events = polling::Events::new();
        let mut next_id = ACCEPT_KEY + 1;
        spawn(move || loop {
            let res: Erm<()> = try {
                events.clear();
                poller.wait(&mut events, None).wrap_err("failed to wait on websocket server poller")?;
                for ev in events.iter() {
                    if ev.key == ACCEPT_KEY {
                        let (sock, client_addr) = listener.accept().wrap_err("failed to accept on socket")?;
                        let id = next_id; next_id += 1;
                        let res = try {
                            unsafe { poller.add(&sock, polling::Event::readable(id))
                                .wrap_err("failed to add socket to poller")?; }
                            let conn = tungstenite::accept(sock).wrap_err("failed to upgrade to websocket")?;
                            clients_ref.lock().unwrap().insert(id, conn);
                        };
                        if let Err(e) = res {
                            log::warn!("error during connection from {}: {}", client_addr, e);
                        };
                        poller.modify(&listener, polling::Event::readable(ACCEPT_KEY))
                            .wrap_err("failed to modify event to poll")?;
                    } else {
                    }
                }
            };
            if let Err(e) = res {
                log::warn!("unhandled error in websocket thread, stopping server: {}", e);
                *channels_ref.lock().unwrap() = None;
                return;
            }
        });
    }
}
