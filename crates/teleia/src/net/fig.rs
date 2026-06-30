use std::io::{Read, Write};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use crate::{Erm, WrapErr};

const KEY: usize = 42;

pub fn read_length_prefixed_utf8<R>(r: &mut R) -> Erm<String> where R: std::io::Read {
    let len = r.read_u32::<LE>()?;
    let mut bs = vec![0; len as usize];
    r.read_exact(&mut bs)?;
    Ok(String::from_utf8(bs)?)
}

#[derive(Debug, Clone)]
pub struct BinaryMessage {
    pub event: Vec<u8>,
    pub data: Vec<u8>
}

pub struct BinaryClient {
    in_buf: Vec<u8>,
    out_buf: Vec<u8>,
    socket: std::net::TcpStream,
    poller: polling::Poller,
}
impl BinaryClient {
    pub fn new(addr: &str, subs: &[&[u8]]) -> Erm<Self> {
        let mut socket = std::net::TcpStream::connect(addr).wrap_err("failed to connect to message bus")?;
        for s in subs {
            write!(socket, "s").wrap_err("failed to send subscribe message to bus")?;
            socket.write_u32::<byteorder::LE>(s.len() as u32).wrap_err("failed to send subscribe message length to bus")?;
            socket.write_all(s).wrap_err("failed to send subscribe message to bus")?;
        }
        socket.set_nonblocking(true).wrap_err("failed to set message bus socket nonblocking")?;
        socket.flush().wrap_err("failed to flush bus connection")?;
        let in_buf = Vec::new();
        let out_buf = Vec::new();
        let poller = polling::Poller::new()?;
        unsafe {
            poller.add(&socket, polling::Event::all(KEY)).wrap_err("failed to add event to poll")?;
        }
        Ok(Self {
            in_buf, out_buf,
            socket,
            poller,
        })
    }
    fn write_length_prefixed(&mut self, buf: &[u8]) -> Erm<()> {
        self.out_buf.write_u32::<byteorder::LE>(buf.len() as u32).wrap_err("failed to send message")?;
        self.out_buf.write_all(buf).wrap_err("failed to send message")?;
        Ok(())
    }
    pub fn publish(&mut self, ev: &[u8], data: &[u8]) -> Erm<()> {
        write!(self.out_buf, "p").wrap_err("failed to send publish message to bus")?;
        self.write_length_prefixed(ev)?;
        self.write_length_prefixed(data)?;
        Ok(())
    }
    pub fn pop_incoming_message(&mut self) -> Option<BinaryMessage> {
        let mut reader = std::io::Cursor::new(&self.in_buf);
        let event_len = reader.read_u32::<byteorder::LE>().ok()?;
        let mut event = vec![0; event_len as usize];
        reader.read_exact(&mut event).ok()?;
        let data_len = reader.read_u32::<byteorder::LE>().ok()?;
        let mut data = vec![0; data_len as usize];
        reader.read_exact(&mut data).ok()?;
        let len = reader.position() as usize;
        self.in_buf.drain(..len);
        Some(BinaryMessage { event, data })
    }
    fn try_read(&mut self) -> Erm<()> {
        let mut buf = [0; 1024];
        match self.socket.read(&mut buf) {
            Ok(sz) => self.in_buf.extend_from_slice(&buf[..sz]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
            e => { e.wrap_err("failed to read from bus socket")?; },
        }
        Ok(())
    }
    fn try_write(&mut self) -> Erm<()> {
        if !self.out_buf.is_empty() {
            match self.socket.write(&self.out_buf) {
                Ok(sz) => { self.out_buf.drain(..sz); },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
                e => { e.wrap_err("failed to write to bus socket")?; },
            }
        }
        Ok(())
    }
    fn resubscribe(&mut self) -> Erm<()> {
        self.poller.modify(&self.socket, polling::Event::readable(KEY)).wrap_err("failed to update event to poll")?;
        if !self.out_buf.is_empty() {
            self.poller.modify(&self.socket, polling::Event::writable(KEY)).wrap_err("failed to update event to poll")?;
        }
        Ok(())
    }
    fn process_events(&mut self, events: &mut polling::Events) -> Erm<()> {
        for ev in events.iter() {
            if ev.key == KEY {
                if ev.readable { self.try_read()?; }
                if ev.writable { self.try_write()?; }
            }
        }
        Ok(())
    }
    pub fn pump(&mut self) -> Erm<()> {
        let mut events = polling::Events::new();
        events.clear();
        self.resubscribe()?;
        self.poller.wait(&mut events, Some(std::time::Duration::from_secs(0)))
            .wrap_err("failed to poll message bus")?;
        self.process_events(&mut events)?;
        Ok(())
    }
    pub fn wait(&mut self) -> Erm<()> {
        let mut events = polling::Events::new();
        events.clear();
        self.resubscribe()?;
        self.poller.wait(&mut events, None)
            .wrap_err("failed to poll message bus")?;
        self.process_events(&mut events)?;
        Ok(())
    }
}
