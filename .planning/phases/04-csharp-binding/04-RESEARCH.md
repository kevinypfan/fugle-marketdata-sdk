# Phase 4: C# Binding Replacement - Research

**Researched:** 2026-01-31
**Domain:** Rust-to-.NET FFI with csbindgen, Task-based async, EventHandler streaming
**Confidence:** MEDIUM

## Summary

This research investigates replacing UniFFI with csbindgen for idiomatic .NET interop. The phase requires transitioning from UniFFI's UDL-based high-level abstractions to csbindgen's low-level extern "C" FFI approach, implementing Task-based async/await patterns, EventHandler-driven WebSocket streaming, and graceful panic recovery at FFI boundaries.

**Key findings:**
- UniFFI and csbindgen serve fundamentally different purposes: UniFFI targets multi-language mobile (Kotlin/Swift) with high-level abstractions; csbindgen is .NET-specific with low-level C FFI focus
- Task-based async requires bridging between Rust's blocking HTTP (ureq) and C#'s async/await using either Task.Run wrapper or TaskCompletionSource callback pattern
- EventHandler pattern is established .NET convention for event-driven APIs, requiring background thread polling with proper synchronization
- Panic recovery via catch_unwind is CRITICAL - unwinding across FFI boundaries causes undefined behavior

**Primary recommendation:** Use spawn_blocking + TaskCompletionSource pattern for async operations. This mirrors the proven Node.js approach (Phase 3) and provides proper cancellation support through CancellationToken.

## Standard Stack

The established libraries/tools for Rust-to-.NET FFI with async support:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| csbindgen | 1.9.x | C# binding generator | .NET-specific, supports function pointers and delegates, official Cysharp tool |
| tokio | 1.49+ | Async runtime | Required for spawn_blocking to bridge sync/async, rt-multi-thread feature needed |
| serde_json | 1.0 | JSON serialization | Type-safe data transfer across FFI boundary |
| thiserror | 2.0 | Error handling | Structured error types for FFI error code conversion |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| libc | 0.2 | C type definitions | For C FFI primitive types (c_char, c_void, etc.) |
| once_cell | 1.19 | Lazy statics | Thread-safe global state (runtime handle, etc.) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| csbindgen | uniffi-bindgen-cs | Third-party UniFFI C# backend vs official .NET-focused tool; csbindgen has better .NET integration |
| TaskCompletionSource | Task.Run wrapper | TCS gives cancellation support and proper async semantics vs simpler Task.Run wrapper |
| EventHandler<T> | IObservable<T> | Standard .NET events vs Reactive Extensions; EventHandler is ubiquitous, IObservable adds dependency |

**Installation:**
```toml
[build-dependencies]
csbindgen = "1.9"

[dependencies]
tokio = { version = "1.49", features = ["rt-multi-thread", "sync", "time"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
libc = "0.2"
once_cell = "1.19"
```

## Architecture Patterns

### Recommended Project Structure
```
cs/
├── src/
│   ├── lib.rs              # csbindgen entry point, extern "C" exports
│   ├── rest_client.rs      # REST API async bridging with TaskCompletionSource
│   ├── websocket.rs        # WebSocket EventHandler with background polling
│   ├── errors.rs           # Error code conversion, panic recovery
│   └── types.rs            # DTO serialization helpers
├── build.rs                # csbindgen invocation, C# code generation
├── Cargo.toml              # Dependencies, crate-type = ["cdylib"]
└── FugleMarketData/
    ├── FugleMarketData.csproj  # Multi-target netstandard2.0;net6.0
    ├── RestClient.cs           # Public async API with Task<T>
    ├── WebSocketClient.cs      # EventHandler<T> pattern
    ├── Exceptions.cs           # Exception hierarchy (FugleException base)
    ├── Models/                 # Record types for responses
    │   ├── Quote.cs
    │   ├── Ticker.cs
    │   └── ...
    └── NativeMethods.cs        # Generated DllImport (from csbindgen)
```

### Pattern 1: Async Bridging with TaskCompletionSource
**What:** Bridge Rust blocking HTTP to C# Task-based async using callback + TCS pattern
**When to use:** All REST API methods requiring async/await support with cancellation
**Example:**
```rust
// Rust FFI boundary - extern "C" callback-based
type ResultCallback = extern "C" fn(user_data: *mut c_void, result_json: *const c_char, error_code: i32);

#[no_mangle]
pub extern "C" fn rest_quote_async(
    client: *const RestClient,
    symbol: *const c_char,
    callback: ResultCallback,
    user_data: *mut c_void
) {
    let result = std::panic::catch_unwind(|| {
        // Clone for move into spawn_blocking
        let client = unsafe { (*client).clone() };
        let symbol = unsafe { CStr::from_ptr(symbol).to_string_lossy().into_owned() };

        // Spawn blocking task
        tokio::task::spawn_blocking(move || {
            match client.stock().intraday().quote().symbol(&symbol).send() {
                Ok(quote) => {
                    let json = serde_json::to_string(&quote).unwrap();
                    let c_json = CString::new(json).unwrap();
                    callback(user_data, c_json.as_ptr(), 0);
                }
                Err(e) => {
                    let error_code = error_to_code(&e);
                    callback(user_data, std::ptr::null(), error_code);
                }
            }
        });
    });

    if result.is_err() {
        // Panic occurred - invoke callback with internal error code
        callback(user_data, std::ptr::null(), ERROR_INTERNAL);
    }
}
```

```csharp
// C# wrapper - TaskCompletionSource pattern
public async Task<Quote> QuoteAsync(string symbol, CancellationToken cancellationToken = default)
{
    var tcs = new TaskCompletionSource<Quote>(TaskCreationOptions.RunContinuationsAsynchronously);

    // Register cancellation
    using var registration = cancellationToken.Register(() =>
        tcs.TrySetCanceled(cancellationToken));

    // GCHandle to prevent premature collection
    var handle = GCHandle.Alloc(tcs);
    try
    {
        NativeMethods.rest_quote_async(
            _handle,
            symbol,
            OnQuoteCallback,
            GCHandle.ToIntPtr(handle)
        );

        return await tcs.Task.ConfigureAwait(false);
    }
    catch
    {
        handle.Free();
        throw;
    }
}

[UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
private static void OnQuoteCallback(IntPtr userData, IntPtr resultJson, int errorCode)
{
    var handle = GCHandle.FromIntPtr(userData);
    var tcs = (TaskCompletionSource<Quote>)handle.Target;
    handle.Free();

    if (errorCode != 0)
    {
        tcs.SetException(ErrorCodeToException(errorCode));
        return;
    }

    var json = Marshal.PtrToStringUTF8(resultJson);
    var quote = JsonSerializer.Deserialize<Quote>(json);
    tcs.SetResult(quote);
}
```

**Why this pattern:**
- Proven in Phase 3 (Node.js spawn_blocking approach)
- CancellationToken support through TCS
- RunContinuationsAsynchronously prevents deadlocks
- GCHandle prevents callback target from being GC'd

### Pattern 2: EventHandler Streaming with Background Polling
**What:** WebSocket events delivered via EventHandler<T> pattern with background thread polling
**When to use:** WebSocket streaming connections requiring .NET event-driven API
**Example:**
```rust
// Rust FFI - message polling
#[no_mangle]
pub extern "C" fn ws_poll_message(
    client: *const WebSocketClient,
    message_json: *mut *mut c_char
) -> i32 {
    std::panic::catch_unwind(|| {
        let client = unsafe { &*client };
        match client.receiver.try_recv() {
            Ok(msg) => {
                let json = serde_json::to_string(&msg).unwrap();
                let c_json = CString::new(json).unwrap().into_raw();
                unsafe { *message_json = c_json; }
                MESSAGE_AVAILABLE
            }
            Err(_) => NO_MESSAGE
        }
    }).unwrap_or(ERROR_INTERNAL)
}
```

```csharp
// C# wrapper - EventHandler pattern
public class WebSocketClient : IAsyncDisposable
{
    private CancellationTokenSource _pollCts;
    private Task _pollTask;

    public event EventHandler<QuoteEventArgs> QuoteReceived;
    public event EventHandler<EventArgs> Connected;
    public event EventHandler<ErrorEventArgs> Error;

    public async Task ConnectAsync(CancellationToken cancellationToken = default)
    {
        // ... connection logic ...

        _pollCts = new CancellationTokenSource();
        _pollTask = Task.Run(() => PollLoop(_pollCts.Token), _pollCts.Token);

        Connected?.Invoke(this, EventArgs.Empty);
    }

    private async Task PollLoop(CancellationToken cancellationToken)
    {
        while (!cancellationToken.IsCancellationRequested)
        {
            try
            {
                var code = NativeMethods.ws_poll_message(_handle, out var messagePtr);

                if (code == MESSAGE_AVAILABLE)
                {
                    var json = Marshal.PtrToStringUTF8(messagePtr);
                    NativeMethods.free_string(messagePtr); // Rust deallocates

                    var quote = JsonSerializer.Deserialize<Quote>(json);
                    QuoteReceived?.Invoke(this, new QuoteEventArgs(quote));
                }
                else if (code == NO_MESSAGE)
                {
                    await Task.Delay(10, cancellationToken).ConfigureAwait(false);
                }
                else
                {
                    Error?.Invoke(this, new ErrorEventArgs(ErrorCodeToException(code)));
                }
            }
            catch (Exception ex)
            {
                Error?.Invoke(this, new ErrorEventArgs(ex));
            }
        }
    }

    public async ValueTask DisposeAsync()
    {
        _pollCts?.Cancel();
        if (_pollTask != null)
            await _pollTask.ConfigureAwait(false);

        // ... cleanup native resources ...
    }
}
```

**Why this pattern:**
- EventHandler<T> is idiomatic .NET event pattern
- Background polling isolates FFI from C# async machinery
- IAsyncDisposable for proper async cleanup
- Error events allow connection error handling

### Pattern 3: Panic Recovery at FFI Boundary
**What:** Catch all panics at extern "C" boundaries to prevent UB
**When to use:** EVERY extern "C" function - mandatory for FFI safety
**Example:**
```rust
// Source: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
use std::panic;
use std::ffi::{CStr, c_char, c_void};

const ERROR_INTERNAL: i32 = -999;

#[no_mangle]
pub extern "C" fn safe_ffi_function(input: *const c_char) -> i32 {
    let result = panic::catch_unwind(|| {
        // All FFI logic inside catch_unwind
        if input.is_null() {
            return ERROR_INVALID_ARG;
        }

        let input_str = unsafe { CStr::from_ptr(input).to_str().unwrap() };
        // ... actual work ...

        SUCCESS
    });

    match result {
        Ok(code) => code,
        Err(_) => ERROR_INTERNAL  // Panic caught, return error code
    }
}
```

**Why this pattern:**
- Unwinding into C/C# is undefined behavior per Rust RFC 2945
- Must catch ALL panics - process abort is unacceptable for library
- Return error codes, let C# convert to FugleInternalException

### Anti-Patterns to Avoid

- **DON'T use extern "C-unwind"**: Only use extern "C" (non-unwinding) for C# FFI. C-unwind is for C++ interop with exception support
- **DON'T call Rust async from C# directly**: Core uses blocking HTTP (ureq). Must bridge with spawn_blocking, not expose raw futures
- **DON'T use Task.Run without ConfigureAwait(false)**: Library code MUST use ConfigureAwait(false) on all awaits to prevent UI thread deadlocks
- **DON'T leak CString pointers**: Every CString::into_raw() requires paired free function. C#'s Marshal.FreeHGlobal won't work - use Rust allocator
- **DON'T expose Arc<T> across FFI**: Use raw pointers with manual ref counting or opaque handles. Arc is Rust-internal, not FFI-safe

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| C# binding generation | Manual DllImport declarations | csbindgen build.rs | Generates type-safe bindings from Rust code, prevents signature mismatches, handles cdecl conventions |
| Async bridging | Custom thread pool + callbacks | TaskCompletionSource + spawn_blocking | Proper CancellationToken support, exception propagation, matches .NET async semantics |
| String marshaling | Manual UTF-8 byte arrays | UnmanagedType.LPUTF8Str (NET 4.7+) | Automatic encoding conversion, prevents encoding bugs, platform-independent |
| Native library packaging | Manual file copy in post-build | NuGet runtimes/{rid}/native/ structure | Automatic RID selection, proper deployment, MSBuild integration |
| Panic handling | Process.Exit on panic | catch_unwind at every FFI boundary | Prevents process termination, converts to C# exceptions, proper error propagation |

**Key insight:** FFI requires meticulous attention to memory ownership, marshaling semantics, and async/threading models. Hand-rolled solutions introduce subtle bugs (memory leaks, data races, undefined behavior). Use proven patterns and established tools.

## Common Pitfalls

### Pitfall 1: Task.Run Without RunContinuationsAsynchronously
**What goes wrong:** TaskCompletionSource default constructor can cause deadlocks when library is consumed from UI thread (WPF, WinForms)
**Why it happens:** Continuations run synchronously on thread that calls SetResult, blocking if that thread is waiting
**How to avoid:** ALWAYS create TCS with `TaskCreationOptions.RunContinuationsAsynchronously` flag
**Warning signs:** Hangs when called from UI thread, works fine from console app

### Pitfall 2: GCHandle Memory Leaks in Callbacks
**What goes wrong:** GCHandle.Alloc without paired Free causes managed object to never be GC'd, memory leak
**Why it happens:** Callbacks are async - exception or early return can skip Free call
**How to avoid:** Use try/finally or let callback itself Free the handle after use
**Warning signs:** Memory usage grows over time, objects not collected despite no references

### Pitfall 3: String Ownership Confusion
**What goes wrong:** Memory corruption or leaks from improper string deallocation across FFI boundary
**Why it happens:** Rust strings allocated with Rust allocator, must be freed by Rust. C#'s Marshal.FreeHGlobal uses C allocator
**How to avoid:** Always provide Rust-side free_string function for any CString returned via into_raw()
**Warning signs:** Crashes in malloc/free, memory leaks visible in Valgrind, random corruption

### Pitfall 4: Missing ConfigureAwait(false) in Library Code
**What goes wrong:** Library methods deadlock when called from UI thread with .Result or .Wait()
**Why it happens:** Await captures SynchronizationContext, continuation queued to UI thread which is blocked
**How to avoid:** Use ConfigureAwait(false) on EVERY await in library code (not application code)
**Warning signs:** Works in async context, deadlocks with synchronous blocking calls

### Pitfall 5: Platform-Specific Path Separators in RID
**What goes wrong:** Native library not found at runtime despite being in NuGet package
**Why it happens:** Incorrect runtimes/ folder structure or wrong RID specification
**How to avoid:** Use exact RID format: `runtimes/win-x64/native/`, `runtimes/linux-x64/native/`, `runtimes/osx-x64/native/`
**Warning signs:** DllNotFoundException at runtime, works in development but not deployment

### Pitfall 6: Not Handling CancellationToken Properly
**What goes wrong:** Operations continue after cancellation, resources not released
**Why it happens:** CancellationToken.Register not used, or callback doesn't check token
**How to avoid:** Register cancellation callback that calls TrySetCanceled on TCS
**Warning signs:** Tasks don't cancel promptly, resources held after cancel

## Code Examples

Verified patterns from official sources and established projects:

### csbindgen Build Script
```rust
// Source: https://github.com/Cysharp/csbindgen (official example)
// build.rs
use csbindgen::Builder;

fn main() {
    // Generate C# bindings from Rust extern "C" functions
    Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("fugle_marketdata")
        .csharp_class_name("NativeMethods")
        .csharp_namespace("Fugle.MarketData.Native")
        .csharp_use_function_pointer(true)  // .NET 5+ function pointers
        .generate_csharp_file("../FugleMarketData/NativeMethods.g.cs")
        .unwrap();

    // Standard Rust library build
    println!("cargo:rerun-if-changed=src/");
}
```

### Multi-Target .csproj for Native Libraries
```xml
<!-- Source: https://learn.microsoft.com/en-us/nuget/create-packages/native-files-in-net-packages -->
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFrameworks>netstandard2.0;net6.0</TargetFrameworks>
    <LangVersion>10.0</LangVersion>
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>

  <!-- Native library references -->
  <ItemGroup>
    <!-- Windows x64 -->
    <Content Include="../target/x86_64-pc-windows-msvc/release/fugle_marketdata.dll">
      <PackagePath>runtimes/win-x64/native</PackagePath>
      <Pack>true</Pack>
      <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
    </Content>

    <!-- Linux x64 -->
    <Content Include="../target/x86_64-unknown-linux-gnu/release/libfugle_marketdata.so">
      <PackagePath>runtimes/linux-x64/native</PackagePath>
      <Pack>true</Pack>
      <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
    </Content>

    <!-- macOS x64 -->
    <Content Include="../target/x86_64-apple-darwin/release/libfugle_marketdata.dylib">
      <PackagePath>runtimes/osx-x64/native</PackagePath>
      <Pack>true</Pack>
      <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
    </Content>
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="System.Text.Json" Version="8.0.0" />
  </ItemGroup>
</Project>
```

### Record Types for Immutable DTOs
```csharp
// Source: https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/records
namespace Fugle.MarketData.Models;

// Positional record - concise syntax, value equality
public record Quote(
    string Symbol,
    decimal LastPrice,
    long Volume,
    DateTime UpdatedAt,
    IReadOnlyList<PriceLevel> Bids,
    IReadOnlyList<PriceLevel> Asks
);

public record PriceLevel(decimal Price, int Size);

// Usage - immutable, thread-safe
var quote = new Quote(
    Symbol: "2330",
    LastPrice: 580.0m,
    Volume: 12345,
    UpdatedAt: DateTime.UtcNow,
    Bids: new List<PriceLevel> { new(579.0m, 100) }.AsReadOnly(),
    Asks: new List<PriceLevel> { new(580.0m, 200) }.AsReadOnly()
);

// Value equality
var quote2 = quote with { LastPrice = 581.0m };
Assert.NotEqual(quote, quote2);  // Different values
```

### IDisposable + IAsyncDisposable Pattern
```csharp
// Source: https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/implementing-disposeasync
public class WebSocketClient : IAsyncDisposable, IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;

        // Async cleanup first
        _pollCts?.Cancel();
        if (_pollTask != null)
        {
            try { await _pollTask.ConfigureAwait(false); }
            catch (OperationCanceledException) { }
        }

        // Synchronous cleanup
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }

    public void Dispose()
    {
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (_disposed) return;

        if (disposing)
        {
            // Managed resources
            _pollCts?.Dispose();
        }

        // Unmanaged resources (always cleanup)
        if (_handle != IntPtr.Zero)
        {
            NativeMethods.ws_close(_handle);
            _handle = IntPtr.Zero;
        }

        _disposed = true;
    }

    ~WebSocketClient() => Dispose(disposing: false);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| UniFFI UDL files | csbindgen extern "C" | Phase 4 migration | Complete API redesign from UDL abstractions to low-level FFI |
| JSON string returns | Typed DTO records | C# 9 (2020) | Type safety, immutability, value equality for free |
| Manual DllImport | csbindgen codegen | csbindgen 1.0 (2022) | Auto-generated bindings prevent signature drift |
| IDisposable only | IAsyncDisposable too | C# 8.0 (2019) | Proper async resource cleanup (WebSocket connections) |
| Task.Factory.StartNew | Task.Run | .NET 4.5 (2012) | Simpler API, better defaults for CPU-bound work |
| UnmanagedType.LPStr | UnmanagedType.LPUTF8Str | .NET Core 1.1 (2016) | Cross-platform UTF-8 string marshaling |

**Deprecated/outdated:**
- UniFFI for C# bindings: Third-party uniffi-bindgen-cs less mature than csbindgen; UniFFI designed for Kotlin/Swift
- TaskCompletionSource without RunContinuationsAsynchronously: .NET 4.6+ mandates this flag to prevent deadlocks
- Synchronous-only APIs: Modern .NET expects async/await for I/O operations

## Open Questions

Things that couldn't be fully resolved:

1. **WebSocket reconnection strategy**
   - What we know: Auto-reconnect enabled by default per CONTEXT.md; exponential backoff is common
   - What's unclear: Specific backoff algorithm (initial delay, max delay, multiplier)
   - Recommendation: Start with 1s initial, 2x multiplier, 60s max - validate with integration tests. Document as configurable.

2. **Optimal polling interval for WebSocket events**
   - What we know: Node.js uses thread-safe function callbacks (no polling); Python uses spawn_blocking + channel polling
   - What's unclear: C# EventHandler pattern requires polling - what interval balances latency vs CPU?
   - Recommendation: Prototype 10ms, 50ms, 100ms intervals. Measure latency vs CPU usage. 10ms likely optimal for real-time quotes.

3. **NuGet package universal binary support (macOS)**
   - What we know: macOS universal2 builds combine x64 and arm64; NuGet RID supports osx-x64, osx-arm64
   - What's unclear: Does NuGet support single universal binary or require separate RID entries?
   - Recommendation: Test with separate osx-x64 and osx-arm64 RIDs. If redundant, investigate osx (non-specific) RID.

4. **csbindgen support for generics/async**
   - What we know: csbindgen generates C# from extern "C" functions (C ABI, no generics); async must be bridged manually
   - What's unclear: Any workarounds or upcoming features for easier async generation?
   - Recommendation: Stick with TaskCompletionSource callback pattern. Monitor csbindgen releases for async improvements.

## Sources

### Primary (HIGH confidence)
- [csbindgen GitHub repository](https://github.com/Cysharp/csbindgen) - Official binding generator documentation
- [Microsoft Learn: Async/Await Best Practices](https://learn.microsoft.com/en-us/archive/msdn-magazine/2013/march/async-await-best-practices-in-asynchronous-programming) - ConfigureAwait guidance
- [Microsoft Learn: IAsyncDisposable](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/implementing-disposeasync) - Async disposal pattern
- [Rust std::panic::catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) - Panic recovery at FFI boundaries
- [Microsoft Learn: NuGet Native Files](https://learn.microsoft.com/en-us/nuget/create-packages/native-files-in-net-packages) - RID-based native packaging

### Secondary (MEDIUM confidence)
- [Medium: csbindgen design philosophy](https://neuecc.medium.com/csbindgen-generate-c-native-code-bridge-automatically-or-modern-approaches-to-native-code-78d9f9a616fb) - Comparison with other approaches
- [TaskCompletionSource best practices](https://medium.com/the-pragmatic-tech-review/asynchronous-programming-in-net-understanding-taskcompletionsource-599c6fe47537) - RunContinuationsAsynchronously requirement
- [C# Record Types for DTOs](https://medium.com/c-sharp-programming/c-tip-use-records-for-dtos-4e27ed7291fa) - Immutable DTO patterns
- [EventHandler threading](https://learn.microsoft.com/en-us/dotnet/standard/asynchronous-programming-patterns/event-based-asynchronous-pattern-overview) - Event-based async pattern
- Multiple web search results on string marshaling, RID catalog, ConfigureAwait

### Tertiary (LOW confidence)
- Web search results on async_ffi crate (not directly applicable - csbindgen uses different approach)
- Community forum discussions on FFI async patterns (informative but not authoritative)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Microsoft docs, csbindgen is de facto standard for Rust-C# FFI
- Architecture: MEDIUM - Patterns proven in Phase 2/3 but not yet validated for C# specifically
- Pitfalls: HIGH - Well-documented issues from Microsoft guidance and community experience

**Research date:** 2026-01-31
**Valid until:** 60 days (stable ecosystem, .NET and csbindgen mature)
