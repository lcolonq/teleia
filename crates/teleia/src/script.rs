use std::{ffi::c_int, mem::MaybeUninit};

use crate::utils;

const NIL: Value = Value { val: PitValue { data: 0xfff4000000000000 } };

#[derive(Debug)]
pub struct Error {
    pub msg: String
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "teleia script error: {}", self.msg)
    }
}
impl std::error::Error for Error {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PitValue { data: u64 }

#[repr(C)]
struct PitRuntime { _data: () }

#[repr(C)]
struct PitLexer {
    input: *const u8,
    len: i64,
    start: i64, end: i64,
    line: i64, column: i64,
    start_line: i64, start_column: i64,
    error: *const u8,
}

#[repr(C)]
struct PitParserTokenInfo {
    token: c_int,
    start: i64, end: i64,
    line: i64, column: i64,
}

#[repr(C)]
struct PitParser {
    lexer: *mut PitLexer,
    cur: PitParserTokenInfo, next: PitParserTokenInfo,
}

unsafe extern "C" {
    pub fn pit_runtime_test(out: *mut u8, out_len: i64, buf: *mut u8, len: i64) -> c_int;
    fn pit_runtime_new(buf: *mut MaybeUninit<u8>, len: i64) -> *mut PitRuntime;
    fn pit_get_error(buf: *mut PitRuntime) -> PitValue;
    fn pit_install_library_essential(buf: *mut PitRuntime);
    fn pit_install_library_plist(buf: *mut PitRuntime);
    fn pit_install_library_alist(buf: *mut PitRuntime);
    fn pit_lex_bytes(ret: *mut PitLexer, buf: *const u8, len: i64);
    fn pit_parser_from_lexer(ret: *mut PitParser, lex: *mut PitLexer);
    fn pit_parse(rt: *mut PitRuntime, parser: *mut PitParser, eof: *mut bool) -> PitValue;
    fn pit_eval(rt: *mut PitRuntime, v: PitValue) -> PitValue;
    fn pit_dump(rt: *mut PitRuntime, buf: *mut u8, len: i64, v: PitValue, readable: bool) -> i64;
    fn pit_eq(rt: *mut PitRuntime, x: PitValue, y: PitValue) -> bool;
}

pub struct Runtime {
    _buf: Box<[MaybeUninit<u8>]>,
    rt: *mut PitRuntime,
}
impl Runtime {
    pub fn new(sz: usize) -> Self {
        let mut buf = Box::new_uninit_slice(sz);
        let rt = unsafe {
            pit_runtime_new((*buf).as_mut_ptr(), buf.len() as i64)
        };
        unsafe {
            pit_install_library_essential(rt);
            pit_install_library_plist(rt);
            pit_install_library_alist(rt);
        }
        Self {
            _buf: buf,
            rt,
        }
    }
    pub fn error(&mut self) -> utils::Erm<()> {
        let e = unsafe { pit_get_error(self.rt) };
        let ve = Value { val: e };
        if self.eq(ve, NIL) { return Ok(()) };
        Err(Error { msg: self.dump(ve).unwrap_or("<unable to dump>".to_owned()) }.into())
    }
    pub fn parse(&mut self, s: &str) -> utils::Erm<Value> {
        let mut rt = Runtime::new(1024 * 1024 * 1024);
        let lexer = Lexer::from_bytes(s.as_bytes());
        let mut parser = Parser::from_lexer(lexer);
        let expr = parser.parse(&mut rt)?.ok_or(Error { msg: "end of file".to_string() })?;
        let res = rt.eval(expr)?;
        Ok(res)
    }
    pub fn eval(&mut self, v: Value) -> utils::Erm<Value> {
        unsafe {
            let ret = pit_eval(self.rt, v.val);
            self.error()?;
            Ok(Value { val: ret })
        }
    }
    pub fn dump(&mut self, v: Value) -> utils::Erm<String> {
        let mut buf = vec![0; 1024];
        unsafe {
            let len = pit_dump(self.rt, buf.as_mut_ptr(), buf.len() as i64, v.val, false) as usize;
            Ok(str::from_utf8(&buf[0..len]).map(|s| s.to_owned())?)
        }
    }
    pub fn eq(&mut self, x: Value, y: Value) -> bool {
        unsafe { pit_eq(self.rt, x.val, y.val) }
    }
}

pub struct Lexer {
    lexer: Box<PitLexer>,
}
impl Lexer {
    pub fn from_bytes(bs: &[u8]) -> Self {
        let mut lexer = Box::new_uninit();
        unsafe {
            pit_lex_bytes(lexer.as_mut_ptr(), bs.as_ptr(), bs.len() as i64);
        }
        Self { lexer: unsafe { lexer.assume_init() } }
    }
}

pub struct Parser {
    _lexer: Lexer,
    parser: Box<PitParser>,
}
impl Parser {
    pub fn from_lexer(mut lexer: Lexer) -> Self {
        let mut parser = Box::new_uninit();
        unsafe {
            pit_parser_from_lexer(parser.as_mut_ptr(), lexer.lexer.as_mut() as *mut _);
        }
        Self {
            _lexer: lexer,
            parser: unsafe { parser.assume_init() }
        }
    }
    pub fn parse(&mut self, rt: &mut Runtime) -> utils::Erm<Option<Value>> {
        unsafe {
            let mut eof = false;
            let val = pit_parse(rt.rt, self.parser.as_mut() as *mut _, &mut eof);
            rt.error()?;
            if eof { return Ok(None) }
            Ok(Some(Value { val }))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Value {
    val: PitValue,
}
