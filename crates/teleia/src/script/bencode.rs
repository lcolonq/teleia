use std::{collections::BTreeMap, io::Write, io::BufRead};

#[derive(Debug)]
pub enum Error {
    Encoding(String),
    Decoding(String),
    IO(std::io::Error),
    Misc(simple_eyre::Report),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encoding(msg) => write!(f, "bencode encoding error: {}", msg),
            Self::Decoding(msg) => write!(f, "bencode decoding error: {}", msg),
            Self::IO(e) => write!(f, "{}", e),
            Self::Misc(e) => write!(f, "{}", e),
        }
    }
}
impl std::error::Error for Error {}
impl Error {
    fn misc(x: impl Into<simple_eyre::Report>) -> Self { Self::Misc(x.into()) }
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Bytestring(Vec<u8>),
    List(Vec<Value>),
    Dictionary(BTreeMap<Vec<u8>, Value>),
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{}", i),
            Self::Bytestring(bs) => if let Ok(s) = str::from_utf8(bs) {
                write!(f, "{:?}", s)
            } else {
                write!(f, "{:?}", bs)
            },
            Self::List(xs) => {
                let mut write_sep = false; 
                f.write_str("[")?;
                for x in xs {
                    if write_sep {
                        f.write_str(" ")?;
                    } else { write_sep = true }
                    write!(f, "{}", x)?;
                }
                f.write_str("]")?;
                Ok(())
            },
            Self::Dictionary(m) => {
                let mut write_sep = false; 
                f.write_str("{")?;
                for (kbs, v) in m.iter() {
                    if write_sep {
                        f.write_str(" ")?;
                    } else { write_sep = true }
                    let s = format!("{:?}", kbs);
                    let k = str::from_utf8(kbs).unwrap_or(&s);
                    write!(f, ":{} {}", k, v)?;
                }
                f.write_str("}")?;
                Ok(())
            },
        }
    }
}
impl Value {
    fn decode_helper<R>(first: u8, r: &mut R) -> Result<Self, Error> where R: BufRead {
        match first {
            b'i' => {
                let mut buf = Vec::new();
                r.read_until(b'e', &mut buf).map_err(Error::IO)?;
                Ok(Self::Integer(
                    str::from_utf8(&buf[..buf.len()-1]).map_err(Error::misc)?
                        .parse().map_err(Error::misc)?))
            },
            b'l' => {
                let mut ret = Vec::new();
                let mut ch: [u8; 1] = [0];
                loop {
                    r.read_exact(&mut ch).map_err(Error::IO)?;
                    if ch[0] == b'e' { break }
                    ret.push(Self::decode_helper(ch[0], r)?);
                }
                Ok(Self::List(ret))
            },
            b'd' => {
                let mut ret = BTreeMap::new();
                let mut ch: [u8; 1] = [0];
                loop {
                    r.read_exact(&mut ch).map_err(Error::IO)?;
                    if ch[0] == b'e' { break }
                    let vk = Self::decode_helper(ch[0], r)?;
                    if let Self::Bytestring(k) = vk {
                        r.read_exact(&mut ch).map_err(Error::IO)?;
                        let v = Self::decode_helper(ch[0], r)?;
                        ret.insert(k, v);
                    } else {
                        return Err(Error::Decoding(format!("non-bytestring key: {:?}", vk)));
                    }
                }
                Ok(Self::Dictionary(ret))
            },
            _ => {
                let mut buf = Vec::new();
                buf.push(first);
                r.read_until(b':', &mut buf).map_err(Error::IO)?;
                let len: usize = str::from_utf8(&buf[..buf.len()-1]).map_err(Error::misc)?
                    .parse().map_err(Error::misc)?;
                buf = vec![0; len];
                r.read_exact(&mut buf[..]).map_err(Error::IO)?;
                Ok(Self::Bytestring(buf))
            }
        }
    }
    pub fn decode<R>(r: &mut R) -> Result<Self, Error> where R: BufRead {
        let mut ch: [u8; 1] = [0];
        r.read_exact(&mut ch).map_err(Error::IO)?;
        Self::decode_helper(ch[0], r)
    }

    pub fn encode<W>(&self, w: &mut W) -> Result<(), Error> where W: Write {
        match self {
            Self::Integer(i) => w.write_all(format!("i{}e", i).as_bytes()).map_err(Error::IO)?,
            Self::Bytestring(bs) => {
                w.write_all(format!("{}:", bs.len()).as_bytes()).map_err(Error::IO)?;
                w.write_all(bs).map_err(Error::IO)?;
            },
            Self::List(xs) => {
                w.write_all(b"l").map_err(Error::IO)?;
                for x in xs { x.encode(w)?; }
                w.write_all(b"e").map_err(Error::IO)?;
            },
            Self::Dictionary(m) => {
                w.write_all(b"d").map_err(Error::IO)?;
                for (k, v) in m {
                    Self::Bytestring(k.clone()).encode(w)?;
                    v.encode(w)?;
                }
                w.write_all(b"e").map_err(Error::IO)?;
            }
        }
        Ok(())
    }
}
