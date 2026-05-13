pub mod bencode;

#[cfg(not(target_arch = "wasm32"))]
pub mod nrepl;

use std::{ffi::c_int, mem::MaybeUninit};

use crate::utils;

pub const NIL: Value = Value { val: PitValue { data: 0xfff4000000000000 } };

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

type PitNativeFunc = extern "C" fn(rt: *mut PitRuntime, args: PitValue, data: *mut PitNativeFuncData) -> PitValue;
#[repr(C)]
struct PitNativeFuncDataShim { _data: () }
struct PitNativeFuncData {
    f: Box<dyn FnMut(&mut Runtime, Value) -> Value>,
}
extern "C" fn unwrap_nativefunc(rt: *mut PitRuntime, args: PitValue, data: *mut PitNativeFuncData) -> PitValue {
    unsafe {
        let ret = ((*data).f)(&mut Runtime::from_shared(rt), Value { val: args });
        ret.val
    }
}

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

#[allow(dead_code)]
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

    fn pit_eq(x: PitValue, y: PitValue) -> bool;

    fn pit_intern(rt: *mut PitRuntime, buf: *const u8, len: i64) -> PitValue;
    fn pit_set(rt: *mut PitRuntime, sym: PitValue, v: PitValue);
    fn pit_fset(rt: *mut PitRuntime, sym: PitValue, v: PitValue);
    fn pit_get(rt: *mut PitRuntime, sym: PitValue) -> PitValue;
    fn pit_fget(rt: *mut PitRuntime, sym: PitValue) -> PitValue;

    fn pit_array_new(rt: *mut PitRuntime, len: i64) -> PitValue;
    fn pit_array_from_buf(rt: *mut PitRuntime, xs: *const PitValue, len: i64) -> PitValue;
    fn pit_array_len(rt: *mut PitRuntime, arr: PitValue) -> i64;
    fn pit_array_get(rt: *mut PitRuntime, arr: PitValue, idx: i64) -> PitValue;
    fn pit_array_set(rt: *mut PitRuntime, arr: PitValue, idx: i64, v: PitValue) -> PitValue;

    fn pit_cons(rt: *mut PitRuntime, car: PitValue, cdr: PitValue) -> PitValue;
    fn pit_list_len(rt: *mut PitRuntime, xs: PitValue) -> i64;
    fn pit_car(rt: *mut PitRuntime, v: PitValue) -> PitValue;
    fn pit_cdr(rt: *mut PitRuntime, v: PitValue) -> PitValue;
    fn pit_setcar(rt: *mut PitRuntime, v: PitValue, x: PitValue);
    fn pit_setcdr(rt: *mut PitRuntime, v: PitValue, x: PitValue);
    fn pit_append(rt: *mut PitRuntime, xs: PitValue, ys: PitValue) -> PitValue;
    fn pit_reverse(rt: *mut PitRuntime, xs: PitValue) -> PitValue;
    fn pit_contains_eq(rt: *mut PitRuntime, needle: PitValue, haystack: PitValue) -> PitValue;
    fn pit_contains_equal(rt: *mut PitRuntime, needle: PitValue, haystack: PitValue) -> PitValue;
    fn pit_plist_get(rt: *mut PitRuntime, k: PitValue, vs: PitValue) -> PitValue;

    fn pit_lambda(rt: *mut PitRuntime, args: PitValue, body: PitValue) -> PitValue;
    fn pit_nativefunc_new_with_data(rt: *mut PitRuntime, f: PitNativeFunc, data: *mut PitNativeFuncDataShim) -> PitValue;
    fn pit_apply(rt: *mut PitRuntime, f: PitValue, args: PitValue) -> PitValue;
}

pub struct Runtime {
    _buf: Box<[MaybeUninit<u8>]>,
    rt: *mut PitRuntime,
}
impl Runtime {
    fn from_shared(rt: *mut PitRuntime) -> Self {
        Self {
            _buf: Box::new([]),
            rt,
        }
    }
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
        eprintln!("ve: {}", ve.val.data);
        Err(Error { msg: self.dump(ve).unwrap_or("<unable to dump>".to_owned()) }.into())
    }
    pub fn parse(&mut self, s: &str) -> utils::Erm<Value> {
        let lexer = Lexer::from_bytes(s.as_bytes());
        let mut parser = Parser::from_lexer(lexer);
        let expr = parser.parse(self)?.ok_or(Error { msg: "end of file".to_string() })?;
        Ok(expr)
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
    pub fn eq(&self, x: Value, y: Value) -> bool {
        unsafe { pit_eq(x.val, y.val) }
    }
    pub fn intern(&mut self, nm: &str) -> utils::Erm<Value> {
        unsafe {
            let bs = nm.as_bytes();
            let sym = pit_intern(self.rt, bs.as_ptr(), bs.len() as i64);
            self.error()?;
            Ok(Value { val: sym })
        }
    }
    pub fn fset(&mut self, sym: Value, f: Value) -> utils::Erm<()> {
        unsafe {
            pit_fset(self.rt, sym.val, f.val);
            self.error()?;
            Ok(())
        }
    }
    pub fn nativefunc_new<F>(&mut self, f: F) -> utils::Erm<Value>
    where F: FnMut(&mut Runtime, Value) -> Value + 'static {
        unsafe {
            let data = Box::leak(Box::new(PitNativeFuncData {
                f: Box::new(f),
            }));
            let ret = pit_nativefunc_new_with_data(self.rt, unwrap_nativefunc, data as *mut PitNativeFuncData as *mut _);
            self.error()?;
            Ok(Value { val: ret })
        }
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
