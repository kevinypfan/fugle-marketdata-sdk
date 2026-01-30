# Testing Patterns

**Analysis Date:** 2026-01-30

## Test Framework

**Rust:**
- Runner: Built-in `cargo test` with test macro support
- Config: No explicit config file; uses Cargo.toml with `[dev-dependencies]`
- Framework: `tokio-test` for async testing, `criterion` for benchmarks
- Run Commands:
  ```bash
  cargo test                    # Run all unit tests
  cargo test --release          # Run with optimizations
  cargo bench                   # Run benchmarks
  ```

**Python:**
- Runner: Direct script execution with `if __name__ == "__main__"`
- Config: No pytest/unittest framework; manual test functions with assertions
- Framework: Standard library `assert` statements and exception handling
- Run Commands:
  ```bash
  python test_rest.py           # Run REST client tests
  python test_websocket.py      # Run WebSocket tests
  python test_2330.py           # Run specific symbol tests
  python test_ws_stream.py      # Run WebSocket streaming tests
  ```

**JavaScript/Node.js:**
- Runner: Node.js direct script execution
- Config: No explicit test framework configured; manual test structure
- Framework: Console assertions and try-catch error handling
- Run Commands:
  ```bash
  node test_rest.js             # Run REST client tests
  node test_websocket.js        # Run WebSocket tests
  ```

## Test File Organization

**Location:**
- Rust: Co-located with source code in modules using `#[cfg(test)]` blocks
  - Example: `core/src/runtime.rs` has `#[cfg(test)] mod tests { }`
  - Benchmarks in separate `core/benches/` directory: `rest_latency.rs`, `websocket_throughput.rs`
- Python: Separate test files in project root - `py/test_*.py`
  - `test_rest.py` - REST client tests
  - `test_websocket.py` - WebSocket client tests
  - `test_ws_callback.py` - WebSocket callback tests
  - `test_ws_stream.py` - WebSocket streaming tests
  - `test_2330.py` - Symbol-specific tests
- JavaScript: Separate test files in project root - `js/test_*.js`
  - `test_rest.js` - REST client tests
  - `test_websocket.js` - WebSocket client tests

**Naming:**
- Rust: Test functions use `#[test]` attribute, prefixed with `test_`
  - `test_runtime_creation()`, `test_runtime_block_on()`, `test_ffi_create_destroy()`
- Python: Test functions named `test_*()` and grouped in `main()` runner
  - `test_import()`, `test_client_creation()`, `test_error_handling()`
- JavaScript: Test grouped in sections with `console.log()` for output
  - Named test descriptions in log output: "Test 1: Module loading", "Test 2: Client creation"

## Test Structure

**Rust Pattern (from `core/src/runtime.rs`):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_runtime_creation() {
        let runtime = AsyncRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_runtime_block_on() {
        let runtime = AsyncRuntime::new().unwrap();
        let result = runtime.block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "test panic")]
    fn test_panic_boundary() {
        std::panic::panic_any("test panic");
    }
}
```

**Patterns:**
- Imports at module level: `use super::*;` for accessing tested items
- Test setup inline: `let runtime = AsyncRuntime::new().unwrap();`
- Assertions with `assert!()`, `assert_eq!()`, `assert_ne!()`
- Error testing with `unwrap()` or `Result` assertions
- Cleanup: Implicit when values go out of scope (Rust's ownership)
- Optional `#[should_panic]` for panic testing

**Python Pattern (from `test_rest.py`):**
```python
def test_import():
    """Test module can be imported"""
    print("Testing import...")
    assert hasattr(marketdata_py, 'RestClient')
    assert hasattr(marketdata_py, 'MarketDataError')
    print("  Import OK")

def test_error_handling():
    """Test error handling with error_code"""
    print("Testing error handling...")

    assert issubclass(MarketDataError, Exception)
    print(f"  MarketDataError is Exception subclass: True")

    try:
        raise MarketDataError("Test error", 1001)
    except MarketDataError as e:
        assert str(e.args[0]) == "Test error"
        assert e.args[1] == 1001
        print(f"  Error code: {e.args[1]}")

    print("  Error handling OK")

def run_all_tests():
    """Run all tests"""
    print("=" * 60)
    print("marketdata_py REST Client Tests")
    print("=" * 60)

    test_import()
    test_client_creation()
    test_error_handling()

    print("=" * 60)
    print("All tests passed!")
    print("=" * 60)

if __name__ == "__main__":
    run_all_tests()
```

**Patterns:**
- Docstrings explain test purpose
- Print statements for test output and progress
- Standard `assert` statements for validation
- Try-except blocks for error testing
- Main test runner calls individual test functions in sequence

**JavaScript Pattern (from `test_rest.js`):**
```javascript
console.log('=== marketdata-js REST Client Tests ===\n');

// Test 1: Module loading
console.log('Test 1: Module loading');
try {
    console.log('  - RestClient class:', typeof RestClient === 'function' ? 'OK' : 'FAIL');
} catch (e) {
    console.log('  - FAIL:', e.message);
}

// Test 2: Client creation
console.log('\nTest 2: Client creation');
try {
    const client = new RestClient('test-api-key');
    console.log('  - new RestClient():', client ? 'OK' : 'FAIL');
} catch (e) {
    console.log('  - FAIL:', e.message);
}

console.log('\n=== Tests Complete ===');
```

**Patterns:**
- Inline test structure with try-catch blocks
- Console output for test names and results
- Status indicators: "OK" for pass, "FAIL" for failure
- Error message extraction from caught exceptions

## Error Testing

**Rust Pattern:**
```rust
#[test]
#[should_panic(expected = "test panic")]
fn test_panic_boundary() {
    std::panic::panic_any("test panic");
}

// Alternative: Using Result return type
#[test]
fn test_result_error() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = AsyncRuntime::new()?;
    Ok(())
}
```

**Python Pattern:**
```python
def test_error_handling():
    try:
        raise MarketDataError("Test error", 1001)
    except MarketDataError as e:
        assert str(e.args[0]) == "Test error"
        assert e.args[1] == 1001
        print(f"  Error correctly caught")

# Test invalid input raises ValueError
try:
    stock.on("invalid_event", on_message)
    print("  ERROR: Should have raised ValueError")
    return False
except ValueError as e:
    print(f"  Invalid event correctly rejected: {e}")
```

**JavaScript Pattern:**
```javascript
try {
    const result = client.futopt.intraday.products('INVALID_TYPE');
    if (result instanceof Error) {
        const isValidationError = result.message.includes('FUTURE') || result.message.includes('OPTION');
        console.log('  - Invalid type validation:', isValidationError ? 'OK (error object)' : 'FAIL');
    } else {
        console.log('  - Invalid type validation: Unexpected success');
    }
} catch (err) {
    const isValidationError = err.message.includes('FUTURE') || err.message.includes('OPTION');
    console.log('  - Invalid type validation:', isValidationError ? 'OK (thrown)' : 'FAIL');
}
```

## Test Data and Fixtures

**Rust:**
- Inline test data: Created in test functions
- File: `core/src/runtime.rs` uses `Arc` and `AtomicU32` for shared state
  ```rust
  let counter = Arc::new(AtomicU32::new(0));
  ```
- No external fixture files; test data created programmatically

**Python:**
- Hard-coded test credentials in test files
  - File: `test_rest.py` line 50 has base64-encoded test API key
  - File: `test_websocket.py` passes "test-api-key" to client creation
- Fixture pattern: Message list built during callback testing
  ```python
  messages_received = []
  def on_message(msg):
      messages_received.append(msg)
  ```
- No separate fixture files or factory pattern

**JavaScript:**
- Test credentials embedded in test code
- File: `test_rest.js` line 26 uses "test-api-key"
- Mock/stub testing: Type checking instead of actual data
  ```javascript
  console.log('  - RestClient class:', typeof RestClient === 'function' ? 'OK' : 'FAIL');
  ```

## Mocking and Stubs

**Strategy:** Limited mocking; tests validate API surface and type signatures rather than behavior.

**Rust Pattern:**
- No explicit mocking framework used
- Tests use real `AsyncRuntime` instances
- Arc/Mutex pattern for shared state in async tests
- File: `core/src/runtime.rs` demonstrates concurrent task testing

**Python Pattern:**
- No mocking framework (mock/unittest.mock not imported)
- Tests check hasattr() for interface validation
- Callback functions used for event testing
  ```python
  def on_message(msg):
      messages_received.append(msg)
  stock.on("message", on_message)
  ```
- Error type validation: `assert issubclass(MarketDataError, Exception)`

**JavaScript Pattern:**
- No mocking framework used
- Type checking validation: `typeof RestClient === 'function'`
- Interface testing: Check property existence
  ```javascript
  console.log('  - quote method:', typeof stockIntraday.quote === 'function' ? 'OK' : 'FAIL');
  ```

## Test Coverage

**Requirements:** No explicit coverage requirements or threshold configured.

**Approach:**
- Rust: Unit tests co-located with modules using `#[cfg(test)]`
- Python: Manual test scripts without coverage tooling
- JavaScript: Manual test scripts without coverage tooling

**What IS Tested:**
- Module/class loading and initialization
- Client creation with different authentication methods
- API surface: method existence and callable validation
- Error handling and exception types
- Client chain access patterns (client.stock.intraday.quote)
- Callback registration and event handling
- State transitions (connected/disconnected)

**What NOT Tested (by design):**
- Actual network calls to live API (tests use invalid credentials)
- Real WebSocket connection and message streaming
- Performance/latency benchmarks (separate from unit tests)
- Integration with external services

## Test Types

**Unit Tests:**
- Scope: Individual function/module behavior
- Location:
  - Rust: `#[test]` in source files (e.g., `core/src/runtime.rs`)
  - Python: `test_*()` functions in `py/test_*.py`
  - JavaScript: Named test sections in `js/test_*.js`
- Approach: Direct function calls, assertion-based validation

**Integration Tests:**
- Scope: Client creation and API surface validation
- Location: Same as unit tests (no separate integration directory)
- Approach: Tests that create clients and validate chains
- Example from `test_rest.py`:
  ```python
  def test_client_chain():
      client = RestClient(">[REDACTED_API_KEY]==")
      stock = client.stock
      assert stock is not None
      stock_intraday = client.stock.intraday
      assert stock_intraday is not None
  ```

**Benchmarks:**
- Framework: Criterion for Rust
- Location: `core/benches/` directory
- Benchmarks:
  - `rest_latency.rs` - REST API call latency
  - `websocket_throughput.rs` - WebSocket message throughput
- Run: `cargo bench`

## Test Async Patterns

**Rust (from `core/src/runtime.rs`):**
```rust
#[test]
fn test_runtime_spawn_and_await() {
    let runtime = AsyncRuntime::new().unwrap();
    let handle = runtime.spawn(async { "hello" });
    let result = runtime.block_on(handle).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_runtime_handle_multiple_tasks() {
    let runtime = AsyncRuntime::new().unwrap();
    let counter = Arc::new(AtomicU32::new(0));

    let mut handles = vec![];
    for _ in 0..10 {
        let counter_clone = counter.clone();
        let handle = runtime.spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        runtime.block_on(handle).unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 10);
}
```

**Python (from `test_websocket.py`):**
```python
def test_not_connected_error(marketdata_py):
    """Test error when not connected."""
    ws = marketdata_py.WebSocketClient("test-api-key")
    stock = ws.stock

    assert not stock.is_connected(), "Should not be connected"

    try:
        stock.subscribe("trades", "2330")
        print("  ERROR: Should have raised RuntimeError")
        return False
    except RuntimeError as e:
        print(f"  Subscribe correctly fails when not connected: {e}")
```

**JavaScript:**
- No async testing; tests validate synchronous interface properties
- Callback registration tested with lambda functions
  ```javascript
  const stockIntraday = stock.intraday;
  console.log('  - quote method:', typeof stockIntraday.quote === 'function' ? 'OK' : 'FAIL');
  ```

## Common Test Patterns

**Validation Pattern - Rust:**
```rust
let runtime = AsyncRuntime::new().unwrap();
let result = runtime.block_on(async { 42 });
assert_eq!(result, 42);
```

**Validation Pattern - Python:**
```python
assert hasattr(marketdata_py, 'RestClient')
assert issubclass(MarketDataError, Exception)
assert not stock.is_connected()
```

**Validation Pattern - JavaScript:**
```javascript
console.log('  - RestClient class:', typeof RestClient === 'function' ? 'OK' : 'FAIL');
console.log('  - quote method:', typeof stockIntraday.quote === 'function' ? 'OK' : 'FAIL');
```

**Error Testing Pattern - Rust:**
```rust
#[test]
#[should_panic(expected = "test panic")]
fn test_panic_boundary() {
    std::panic::panic_any("test panic");
}
```

**Error Testing Pattern - Python:**
```python
try:
    raise MarketDataError("Test error", 1001)
except MarketDataError as e:
    assert str(e.args[0]) == "Test error"
    assert e.args[1] == 1001
```

**Error Testing Pattern - JavaScript:**
```javascript
try {
    const result = client.stock.intraday.quote('2330');
} catch (err) {
    const hasErrorCode = err.message.match(/\[\d+\]/);
    console.log('    - Error has code format [XXXX]:', hasErrorCode ? 'OK' : 'FAIL');
}
```

---

*Testing analysis: 2026-01-30*
