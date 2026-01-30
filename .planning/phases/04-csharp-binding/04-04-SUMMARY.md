---
phase: 04-csharp-binding
plan: 04
subsystem: wrapper
tags: [csharp, task-async, restclient, exception-hierarchy, record-models]

# Dependency graph
requires:
  - phase: 04-02
    provides: REST client FFI with async callbacks
provides:
  - C# RestClient with Task-based async methods
  - Exception hierarchy (FugleException > ApiException > RateLimitException)
  - Record models with JsonPropertyName for deserialization
  - Sync wrapper methods for legacy compatibility
  - TaskCompletionSource async bridging pattern
affects: [04-05 (tests will use this RestClient wrapper)]

# Tech tracking
tech-stack:
  added: [IsExternalInit 1.0.3, System.Text.Json 8.0.5]
  patterns:
    - "TaskCompletionSource with RunContinuationsAsynchronously"
    - "UnmanagedFunctionPointer delegate for netstandard2.0"
    - "GCHandle for callback user_data marshaling"
    - "GetAwaiter().GetResult() for sync wrappers"

key-files:
  created:
    - cs/FugleMarketData/FugleMarketData.csproj
    - cs/FugleMarketData/Exceptions.cs
    - cs/FugleMarketData/Models/Quote.cs
    - cs/FugleMarketData/Models/Trade.cs
    - cs/FugleMarketData/Models/Ticker.cs
    - cs/FugleMarketData/Models/PriceLevel.cs
    - cs/FugleMarketData/NativeErrorCodes.cs
    - cs/FugleMarketData/RestClient.cs
  modified:
    - cs/build.rs (added all module files to csbindgen scanner)
    - cs/FugleMarketData/NativeMethods.g.cs (regenerated with 170 lines)

key-decisions:
  - "IsExternalInit polyfill for record support in netstandard2.0"
  - "System.Text.Json 8.0.5 (latest stable, fixes vulnerability in 6.0.0)"
  - "UnmanagedFunctionPointer delegate (not UnmanagedCallersOnly) for netstandard2.0"
  - "TaskCreationOptions.RunContinuationsAsynchronously avoids ConfigureAwait(false)"

patterns-established:
  - "C# record models map directly to Rust structs via JSON"
  - "TaskCompletionSource bridges callback-based FFI to async/await"
  - "GCHandle.Alloc/Free for safe callback user_data passing"
  - "Marshal.GetFunctionPointerForDelegate for callback pointers"

# Metrics
duration: 6min
completed: 2026-01-31
---

# Phase 4 Plan 04: C# Wrapper Layer Summary

**Multi-target C# project with exception hierarchy, record models, and RestClient implementing Task-based async pattern**

## Performance

- **Duration:** 6 min
- **Started:** 2026-01-30T20:15:15Z
- **Completed:** 2026-01-30T20:20:44Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- C# project targets netstandard2.0 and net6.0
- Exception hierarchy: FugleException > ApiException > RateLimitException, plus AuthException, ConnectionException, FugleInternalException
- Record models: Quote (60 properties), Trade, TradesResponse, Ticker (30 properties), PriceLevel, TotalStats, TradeInfo, TradingHalt
- RestClient with 3 async methods (GetStockQuoteAsync, GetStockTradesAsync, GetStockTickerAsync) and 3 sync wrappers
- TaskCompletionSource bridges FFI callbacks to C# async/await
- CancellationToken support on all async methods
- Proper IDisposable with native handle cleanup

## Task Commits

Each task was committed atomically:

1. **Task 1: Create C# project structure with exceptions and models** - `93f14d2` (feat)
2. **Task 2: Implement RestClient with async/sync methods** - `05dd6ba` (feat)

## Files Created/Modified

**Task 1:**
- `cs/FugleMarketData/FugleMarketData.csproj` - Multi-target netstandard2.0;net6.0, IsExternalInit polyfill
- `cs/FugleMarketData/Exceptions.cs` - Exception hierarchy with ErrorCodeMapper
- `cs/FugleMarketData/Models/Quote.cs` - 60 properties with JsonPropertyName
- `cs/FugleMarketData/Models/Trade.cs` - Trade and TradesResponse records
- `cs/FugleMarketData/Models/Ticker.cs` - 30 properties with JsonPropertyName
- `cs/FugleMarketData/Models/PriceLevel.cs` - PriceLevel, TradeInfo, TotalStats, TradingHalt

**Task 2:**
- `cs/build.rs` - Added all module files to csbindgen scanner
- `cs/FugleMarketData/NativeMethods.g.cs` - Regenerated with 14 FFI functions
- `cs/FugleMarketData/NativeErrorCodes.cs` - Error code constants
- `cs/FugleMarketData/RestClient.cs` - 340 lines with TaskCompletionSource pattern

## Decisions Made

**1. IsExternalInit polyfill for netstandard2.0**
- Rationale: C# records require IsExternalInit, not available in netstandard2.0
- Solution: IsExternalInit NuGet package 1.0.3
- Impact: Enables modern record syntax across all target frameworks

**2. System.Text.Json 8.0.5 (not 6.0.0)**
- Rationale: 6.0.0 has known high-severity vulnerability (GHSA-8g4q-xg66-9fp4)
- Solution: Upgrade to 8.0.5 (latest stable)
- Impact: Secure JSON deserialization

**3. UnmanagedFunctionPointer delegate pattern**
- Rationale: UnmanagedCallersOnly requires .NET 5+, not available in netstandard2.0
- Solution: Marshal.GetFunctionPointerForDelegate with static delegate field
- Impact: Cross-framework callback support

**4. TaskCreationOptions.RunContinuationsAsynchronously**
- Rationale: Avoid ConfigureAwait(false) boilerplate on all awaits
- Solution: Set on TaskCompletionSource constructor
- Impact: Clean async code, proper continuation behavior

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] IsExternalInit missing in netstandard2.0**
- **Found during:** Task 1 (Build after creating record models)
- **Issue:** Compiler error CS0518: IsExternalInit not found in netstandard2.0
- **Fix:** Added IsExternalInit 1.0.3 NuGet package with PrivateAssets=all
- **Files modified:** cs/FugleMarketData/FugleMarketData.csproj
- **Verification:** dotnet build succeeds for both netstandard2.0 and net6.0
- **Committed in:** 93f14d2 (Task 1 commit)

**2. [Rule 1 - Bug] System.Text.Json 6.0.0 has security vulnerability**
- **Found during:** Task 1 (NuGet restore warnings)
- **Issue:** NU1903 warning: System.Text.Json 6.0.0 has high-severity vulnerability
- **Fix:** Upgraded to System.Text.Json 8.0.5 (latest stable, no warnings)
- **Files modified:** cs/FugleMarketData/FugleMarketData.csproj
- **Verification:** dotnet build has 0 warnings
- **Committed in:** 93f14d2 (Task 1 commit)

**3. [Rule 3 - Blocking] UnmanagedCallersOnly not available in netstandard2.0**
- **Found during:** Task 2 (Build after creating RestClient)
- **Issue:** Compiler error CS0246: UnmanagedCallersOnly requires .NET 5+
- **Fix:** Changed to UnmanagedFunctionPointer delegate with Marshal.GetFunctionPointerForDelegate
- **Files modified:** cs/FugleMarketData/RestClient.cs
- **Verification:** dotnet build succeeds for netstandard2.0
- **Committed in:** 05dd6ba (Task 2 commit)

**4. [Rule 2 - Missing Critical] csbindgen not scanning all modules**
- **Found during:** Task 2 (NativeMethods.g.cs only had fugle_version)
- **Issue:** build.rs only scanned lib.rs, missing rest_client/websocket FFI functions
- **Fix:** Added .input_extern_file() for all modules to csbindgen Builder
- **Files modified:** cs/build.rs
- **Verification:** NativeMethods.g.cs regenerated with 170 lines (14 FFI functions)
- **Committed in:** 05dd6ba (Task 2 commit)

---

**Total deviations:** 4 auto-fixed (2 bugs, 1 missing critical, 1 blocking)
**Impact on plan:** All deviations were essential fixes for compilation and security. No scope creep.

## Issues Encountered

None - plan executed smoothly after auto-fixing compilation issues.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

C# wrapper layer complete. Ready for:
- 04-05: C# test suite and examples
- Phase 5: Distribution (NuGet packaging)

RestClient provides idiomatic .NET API:
- async/await with CancellationToken
- sync methods for legacy compatibility
- Proper exception hierarchy matching core errors
- IDisposable for resource cleanup

---
*Phase: 04-csharp-binding*
*Completed: 2026-01-31*
