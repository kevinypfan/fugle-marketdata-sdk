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

// Helper function to invoke callback from error handling path
unsafe fn invoke_callback(callback_addr: usize, user_data_addr: usize, error_code: c_int) {
    let callback: ResultCallback = std::mem::transmute(callback_addr);
    let user_data = user_data_addr as *mut c_void;
    callback(user_data, ptr::null(), error_code);
}

// Helper function to invoke callback with result
unsafe fn invoke_callback_with_result(callback_addr: usize, user_data_addr: usize, json: *const c_char, error_code: c_int) {
    let callback: ResultCallback = std::mem::transmute(callback_addr);
    let user_data = user_data_addr as *mut c_void;
    callback(user_data, json, error_code);
}

// ============================================================================
// Stock Intraday Endpoints
// ============================================================================

/// Get intraday quote for a stock symbol (async)
#[no_mangle]
pub extern "C" fn fugle_rest_stock_quote_async(
    handle: *const RestClientHandle,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;
        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.stock().intraday().quote().symbol(&symbol).send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}

/// Get intraday trades for a stock symbol (async)
#[no_mangle]
pub extern "C" fn fugle_rest_stock_trades_async(
    handle: *const RestClientHandle,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;
        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.stock().intraday().trades().symbol(&symbol).send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}

/// Get intraday ticker info for a stock symbol (async)
#[no_mangle]
pub extern "C" fn fugle_rest_stock_ticker_async(
    handle: *const RestClientHandle,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;
        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.stock().intraday().ticker().symbol(&symbol).send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}

/// Get intraday candles for a stock symbol (async)
#[no_mangle]
pub extern "C" fn fugle_rest_stock_candles_async(
    handle: *const RestClientHandle,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;
        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.stock().intraday().candles().symbol(&symbol).send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}

/// Get intraday volumes for a stock symbol (async)
#[no_mangle]
pub extern "C" fn fugle_rest_stock_volumes_async(
    handle: *const RestClientHandle,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;
        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.stock().intraday().volumes().symbol(&symbol).send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}

// ============================================================================
// FutOpt Intraday Endpoints
// ============================================================================

/// Get intraday quote for a FutOpt contract (async)
#[no_mangle]
pub extern "C" fn fugle_rest_futopt_quote_async(
    handle: *const RestClientHandle,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;
        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.futopt().intraday().quote().symbol(&symbol).send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}

/// Get intraday ticker info for a FutOpt contract (async)
#[no_mangle]
pub extern "C" fn fugle_rest_futopt_ticker_async(
    handle: *const RestClientHandle,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;
        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.futopt().intraday().ticker().symbol(&symbol).send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}

/// Get available FutOpt products (async)
#[no_mangle]
pub extern "C" fn fugle_rest_futopt_products_async(
    handle: *const RestClientHandle,
    callback: ResultCallback,
    user_data: *mut c_void,
) {
    let callback_addr = callback as usize;
    let user_data_addr = user_data as usize;

    let result = catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let client = unsafe { &(*handle).client };
        let client_clone = client.clone();

        RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                client_clone.futopt().intraday().products().send()
            }).await;

            match result {
                Ok(Ok(data)) => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let c_json = string_to_cstring(&json);
                    unsafe { invoke_callback_with_result(callback_addr, user_data_addr, c_json, SUCCESS) };
                }
                Ok(Err(e)) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, error_to_code(&e)) };
                }
                Err(_) => {
                    unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
                }
            }
        });

        Ok(())
    }));

    if result.is_err() {
        unsafe { invoke_callback(callback_addr, user_data_addr, ERROR_INTERNAL) };
    }
}
