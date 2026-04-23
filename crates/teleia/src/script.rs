use std::ffi::c_int;

unsafe extern "C" {
    pub fn pit_runtime_test(out: *mut u8, out_len: i64, buf: *mut u8, len: i64) -> c_int;
}
