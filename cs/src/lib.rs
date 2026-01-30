//! C# bindings for marketdata-core via csbindgen

mod errors;
mod rest_client;
mod types;
mod websocket;

pub use errors::*;
pub use rest_client::*;
pub use types::*;
pub use websocket::*;

use std::ffi::c_char;

/// Get library version (stub for csbindgen to generate bindings)
#[no_mangle]
pub extern "C" fn fugle_version() -> *const c_char {
    // Static string literal with stable address
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}
