//! FFI type helpers for C# interop

use std::ffi::{CStr, CString, c_char};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

/// Global tokio runtime for async operations
pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
});

/// Convert C string to Rust String safely
/// Returns None if pointer is null or invalid UTF-8
pub unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
}

/// Convert Rust string to C string, returning raw pointer
/// Caller must call fugle_free_string() to deallocate
pub fn string_to_cstring(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string allocated by Rust (for C# to call)
#[no_mangle]
pub extern "C" fn fugle_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
