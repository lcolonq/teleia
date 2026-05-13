use super::bencode;
use crate::{utils, Erm, WrapErr};

use std::io::{BufReader, Read, Write};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

const ACCEPT_KEY: usize = 1;

type ClientId = usize;
type ClientMessage = (ClientId, bencode::Value);
struct ClientConnection {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}
impl ClientConnection {
    fn new(s: TcpStream) -> Erm<Self> {
        let reader = BufReader::new(s.try_clone()?);
        Ok(Self {
            writer: s,
            reader,
        })
    }
}

pub struct Server {
    clients: Arc<Mutex<HashMap<ClientId, ClientConnection>>>,
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
        let recv_clients_ref = self.clients.clone();
        let send_clients_ref = self.clients.clone();
        let (outgoing_sender, outgoing_receiver) = channel();
        let (incoming_sender, incoming_receiver) = channel();
        *self.channels.lock().unwrap() = Some((outgoing_sender, incoming_receiver));
        let channels_ref = self.channels.clone();
        let listener = TcpListener::bind(addr).expect("failed to bind server socket");
        let poller = polling::Poller::new().expect("failed to create poller for nREPL server");
        unsafe {
            poller.add(&listener, polling::Event::readable(ACCEPT_KEY))
                .expect("failed to add poll server socket");
        }
        let mut events = polling::Events::new();
        let mut next_id = ACCEPT_KEY + 1;
        spawn(move || loop {
            // write all outgoing messages
            while let Ok((cid, msg)) = outgoing_receiver.recv() {
                let res: Erm<()> = try {
                    if let Some(sock) = send_clients_ref.lock().unwrap().get_mut(&cid) {
                        let res = msg.encode(&mut sock.writer);
                        match res {
                            Ok(_) => {},
                            Err(bencode::Error::IO(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                log::info!("client {} receive send block", cid);
                            },
                            Err(e) => utils::erm(e)?,
                        }
                    }
                };
                if let Err(e) = res {
                    log::warn!("error when sending to client {}, disconnecting: {}", cid, e);
                    send_clients_ref.lock().unwrap().remove(&cid);
                }
            }
        });
        spawn(move || loop {
            // wait for and receive all new connections and incoming messages
            let res: Erm<()> = try {
                events.clear();
                poller.wait(&mut events, None).wrap_err("failed to wait on nREPL server poller")?;
                for ev in events.iter() {
                    if ev.key == ACCEPT_KEY {
                        let (sock, client_addr) = listener.accept().wrap_err("failed to accept on socket")?;
                        let id = next_id; next_id += 1;
                        let res: Erm<()> = try {
                            unsafe { poller.add(&sock, polling::Event::readable(id))
                                .wrap_err("failed to add socket to poller")?; }
                            recv_clients_ref.lock().unwrap().insert(id, ClientConnection::new(sock)?);
                            log::info!("new connected client! {}", id);
                        };
                        if let Err(e) = res {
                            log::warn!("error during connection from {}: {}", client_addr, e);
                        };
                        poller.modify(&listener, polling::Event::readable(ACCEPT_KEY))
                            .wrap_err("failed to modify event to poll")?;
                    } else {
                        let res: Erm<()> = try {
                            if let Some(sock) = recv_clients_ref.lock().unwrap().get_mut(&ev.key) {
                                match bencode::Value::decode(&mut sock.reader) {
                                    Ok(v) => incoming_sender.send((ev.key, v))
                                        .wrap_err("failed to send incoming message on channel")?,
                                    Err(e) => utils::erm(e)?,
                                }
                                poller.modify(&sock.writer, polling::Event::readable(ev.key))
                                    .wrap_err("failed to modify event to poll")?;
                            } else {
                                log::warn!("poller indicated event for unknown client: {}", ev.key);
                            }
                        };
                        if let Err(e) = res {
                            log::warn!("error on client {} connection, disconnecting: {}", ev.key, e);
                            recv_clients_ref.lock().unwrap().remove(&ev.key);
                        }
                    }
                }
            };
            if let Err(e) = res {
                log::warn!("unhandled error in nREPL thread, stopping server: {}", e);
                *channels_ref.lock().unwrap() = None;
                return;
            }
        });
    }
    pub fn clients(&self) -> Vec<ClientId> {
        self.clients.lock().unwrap().keys().copied().collect()
    }
    pub fn poll(&mut self) -> Option<(ClientId, bencode::Value)> {
        if let Some((_, incoming)) = &*self.channels.lock().unwrap() {
            incoming.try_recv().ok()
        } else { None }
    }
    pub fn send(&self, client: ClientId, msg: bencode::Value) {
        if let Some((outgoing, _)) = &mut *self.channels.lock().unwrap() {
            if let Err(e) = outgoing.send((client, msg)) {
                log::warn!("failed to send nREPL message to client {} on channel: {}", client, e);
            }
        } else {
            log::warn!("tried to send message to client {}, but nREPL is not connected", client);
        }
    }
}
