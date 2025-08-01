use std::io::{BufRead, Read, Write};
use byteorder::WriteBytesExt;

#[derive(Debug, Clone)]
pub struct SexpMessage {
    pub event: lexpr::Value,
    pub data: lexpr::Value,
}

pub struct SexpClient {
    reader: std::io::BufReader<std::net::TcpStream>,
    buf: String,
}
impl SexpClient {
    pub fn new(addr: &str, subs: &[lexpr::Value]) -> Self {
        let mut socket = std::net::TcpStream::connect(addr).expect("failed to connect to message bus");
        socket.set_nonblocking(true).expect("failed to set message bus socket nonblocking");
        for s in subs {
            write!(socket, "(sub {})\n", s).expect("failed to send subscribe message to bus");
        }
        let reader = std::io::BufReader::new(socket);
        Self { reader, buf: String::new(), }
    }
    pub fn pump(&mut self) -> Option<SexpMessage> {
        match self.reader.read_line(&mut self.buf) {
            Ok(l) => {
                // log::info!("read line: {}", self.buf);
                let mv = lexpr::from_str(&self.buf);
                self.buf.clear();
                match mv {
                    Ok(v) => {
                        match v.as_cons() {
                            Some(cs) => {
                                Some(SexpMessage { event: cs.car().clone(), data: cs.cdr().clone() })
                            },
                            _ => { log::error!("malformed message bus input s-expression: {}", v); None },
                        }
                    },
                    Err(e) => { log::error!("malformed message bus input line: {}", e); None },
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // if self.buf.len() > 0 {
                //     log::error!("error wouldblock: buf is {}", self.buf);
                // }
                None
            },
            Err(e) => panic!("IO error on message bus: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryMessage {
    pub event: Vec<u8>,
    pub data: Vec<u8>
}
#[derive(Debug, Clone)]
pub enum BinaryClientState {
    PartialEventLength { buf_len: usize, buf: [u8; 4] },
    PartialEvent { len: usize, buf_len: usize, buf: Vec<u8> },
    PartialDataLength { event: Vec<u8>, buf_len: usize, buf: [u8; 4] },
    PartialData { event: Vec<u8>, len: usize, buf_len: usize, buf: Vec<u8> },
    Message { event: Vec<u8>, data: Vec<u8> },
}
impl Default for BinaryClientState {
    fn default() -> Self {
        Self::PartialEventLength { buf_len: 0, buf: [0; 4] }
    }
}
pub struct BinaryClient {
    state: BinaryClientState,
    writer: std::net::TcpStream,
    reader: std::io::BufReader<std::net::TcpStream>,
}
impl BinaryClient {
    pub fn new(addr: &str, blocking: bool, subs: &[&[u8]]) -> Self {
        let mut socket = std::net::TcpStream::connect(addr).expect("failed to connect to message bus");
        socket.set_nonblocking(!blocking).expect("failed to set message bus socket nonblocking");
        for s in subs {
            write!(socket, "s").expect("failed to send subscribe message to bus");
            socket.write_u32::<byteorder::LE>(s.len() as u32).expect("failed to send subscribe message length to bus");
            socket.write_all(s).expect("failed to send subscribe message to bus");
        }
        socket.flush().expect("failed to flush bus connection");
        let writer = socket.try_clone().expect("failed to clone socket");
        let reader = std::io::BufReader::new(socket);
        Self {
            state: BinaryClientState::PartialEventLength { buf_len: 0, buf: [0; 4] },
            writer,
            reader,
        }
    }
    fn write_length_prefixed(&mut self, buf: &[u8]) {
        self.writer.write_u32::<byteorder::LE>(buf.len() as u32).expect("failed to send message");
        self.writer.write_all(buf).expect("failed to send message");
        
    }
    pub fn publish(&mut self, ev: &[u8], data: &[u8]) {
        write!(self.writer, "p").expect("failed to send publish message to bus");
        self.write_length_prefixed(ev);
        self.write_length_prefixed(data);
    }
    fn read(reader: &mut std::io::BufReader<std::net::TcpStream>, buf: &mut [u8]) -> Option<usize> {
        match reader.read(buf) {
            Ok(sz) => Some(sz),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                None
            },
            Err(e) => panic!("IO error on message bus: {}", e),
        }
    }
    fn update_state(
        reader: &mut std::io::BufReader<std::net::TcpStream>,
        mut state: BinaryClientState,
    ) -> BinaryClientState {
        loop {
            state = match state {
                BinaryClientState::PartialEventLength { mut buf_len, mut buf } => {
                    buf_len += if let Some(x) = Self::read(reader, &mut buf[buf_len..]) {
                        x
                    } else { break BinaryClientState::PartialEventLength { buf_len, buf }; };
                    if buf_len == 4 {
                        let len = u32::from_le_bytes(buf) as usize;
                        BinaryClientState::PartialEvent {
                            len,
                            buf_len: 0,
                            buf: vec![0; len],
                        }
                    } else { BinaryClientState::PartialEventLength { buf_len, buf } }
                },
                BinaryClientState::PartialEvent { len, mut buf_len, mut buf } => {
                    buf_len += if let Some(x) = Self::read(reader, &mut buf[buf_len..]) {
                        x
                    } else { break BinaryClientState::PartialEvent { len, buf_len, buf }; };
                    if buf_len == len {
                        BinaryClientState::PartialDataLength {
                            event: buf.clone(),
                            buf_len: 0,
                            buf: [0; 4],
                        }
                    } else { BinaryClientState::PartialEvent { len, buf_len, buf } }
                },
                BinaryClientState::PartialDataLength { event, mut buf_len, mut buf } => {
                    buf_len += if let Some(x) = Self::read(reader, &mut buf[buf_len..]) {
                        x
                    } else { break BinaryClientState::PartialDataLength { event, buf_len, buf }; };
                    if buf_len == 4 {
                        let len = u32::from_le_bytes(buf) as usize;
                        BinaryClientState::PartialData {
                            event,
                            len,
                            buf_len: 0,
                            buf: vec![0; len],
                        }
                    } else { BinaryClientState::PartialDataLength { event, buf_len, buf } }
                },
                BinaryClientState::PartialData { event, len, mut buf_len, mut buf } => {
                    buf_len += if let Some(x) = Self::read(reader, &mut buf[buf_len..]) {
                        x
                    } else { break BinaryClientState::PartialData { event, len, buf_len, buf }; };
                    if buf_len == len {
                        BinaryClientState::Message {
                            event,
                            data: buf.clone(),
                        }
                    } else { BinaryClientState::PartialData { event, len, buf_len, buf } }
                },
                st@BinaryClientState::Message{..} => break st,
            };
        }
    }
    pub fn pump(&mut self) -> Option<BinaryMessage> {
        self.state = Self::update_state(&mut self.reader, std::mem::take(&mut self.state));
        match std::mem::take(&mut self.state) {
            BinaryClientState::Message { event, data } => {
                self.state = BinaryClientState::PartialEventLength { buf_len: 0, buf: [0; 4] };
                Some(BinaryMessage {
                    event: event,
                    data: data,
                })
            },
            st => {
                self.state = st;
                None
            }
        }
    }
}
