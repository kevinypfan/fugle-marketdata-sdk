//! REST client FFI exports for C# binding

use std::ffi::{c_char, c_void, c_int};
use std::panic::AssertUnwindSafe;
use std::ptr;

use marketdata_core::{RestClient as CoreRestClient, Auth};

use crate::errors::{catch_panic, error_to_code, SUCCESS, ERROR_INVALID_ARG, ERROR_INTERNAL};
use crate::types::{cstr_to_string, string_to_cstring, RUNTIME};

/// Opaque handle for REST client
pub struct RestClientHandle {
    client: CoreRestClient,
}

/// Callback type for async REST operations
/// Parameters: user_data, result_json (null on error), error_code (0 = success)
pub type ResultCallback = extern "C" fn(
    user_data: *mut c_void,
    result_json: *const c_char,
    error_code: c_int,
);

/// Create a new REST client with API key
/// Returns: client handle pointer, or null on failure
#[no_mangle]
pub extern "C" fn fugle_rest_client_new(api_key: *const c_char) -> *mut RestClientHandle {
    catch_panic(AssertUnwindSafe(|| {
        let api_key = unsafe { cstr_to_string(api_key) }
            .ok_or(ERROR_INVALID_ARG)?;

        let auth = Auth::ApiKey(api_key);
        let client = CoreRestClient::new(auth);

        let handle = Box::new(RestClientHandle { client });
        Ok(Box::into_raw(handle))
    }))
    .unwrap_or(ptr::null_mut())
}

/// Destroy a REST client handle
#[no_mangle]
pub extern "C" fn fugle_rest_client_free(handle: *mut RestClientHandle) {
    if !handle.is_null() {
        let _ = catch_panic(AssertUnwindSafe(|| {
            unsafe {
                let _ = Box::from_raw(handle);
            }
            Ok(())
        }));
    }
}
