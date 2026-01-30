---
phase: 04-csharp-binding
plan: 01
subsystem: ffi
tags: [csbindgen, rust, csharp, ffi, cdylib, error-handling, panic-recovery]

# Dependency graph
requires:
  - phase: 01-workspace-migration
    provides: Unified workspace structure with core, py, js, uniffi crates
provides:
  - cs/ crate with csbindgen build infrastructure
  - Error code conversion for all MarketDataError variants
  - Panic recovery pattern at FFI boundaries
  - Global tokio runtime for async operations
  - String marshaling helpers for C# interop
affects: [04-csharp-binding (all remaining plans depend on this FFI foundation)]

# Tech tracking
tech-stack:
  added: [csbindgen 1.9, libc 0.2, once_cell 1.19]
  patterns: [FFI error code conversion, panic recovery with catch_unwind, string ownership via fugle_free_string]

key-files:
  created:
    - cs/Cargo.toml
    - cs/build.rs
    - cs/src/lib.rs
    - cs/src/errors.rs
    - cs/src/types.rs
    - cs/FugleMarketData/NativeMethods.g.cs (generated)
  modified:
    - Cargo.toml (workspace members and dependencies)

key-decisions:
  - "Use csbindgen over UniFFI for .NET-specific FFI generation"
  - "Error codes map to MarketDataError variants via negative integers"
  - "catch_unwind wraps all FFI boundaries to prevent process abort"
  - "Global tokio RUNTIME for async operation bridging"

patterns-established:
  - "FFI error handling: Rust Result → error code → C# exception conversion"
  - "String ownership: Rust allocates via CString::into_raw(), C# frees via fugle_free_string()"
  - "Panic safety: All extern C functions must use catch_unwind pattern"

# Metrics
duration: 2min
completed: 2026-01-31
---

# Phase 4 Plan 01: C# Binding Foundation Summary

**csbindgen FFI infrastructure with error code conversion, panic recovery, and string marshaling for .NET interop**

## Performance

- **Duration:** 2 min
- **Started:** 2026-01-30T20:02:33Z
- **Completed:** 2026-01-30T20:05:01Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- cs/ crate compiles as cdylib with csbindgen generating NativeMethods.g.cs
- Error codes defined for all 12 MarketDataError variants with conversion function
- catch_panic helper provides panic recovery pattern for FFI safety
- Global tokio runtime initialized for async operation bridging
- String marshaling helpers (cstr_to_string, string_to_cstring, fugle_free_string) for C# interop

## Task Commits

Each task was committed atomically:

1. **Task 1: Create cs/ crate with csbindgen build infrastructure** - `3ae3389` (feat)
2. **Task 2: Implement error codes and panic recovery foundation** - `2f0211a` (feat)

## Files Created/Modified
- `Cargo.toml` - Added cs/ to workspace members, added csbindgen/libc/once_cell dependencies
- `cs/Cargo.toml` - C# binding crate with cdylib target, core dependencies
- `cs/build.rs` - csbindgen invocation to generate NativeMethods.g.cs
- `cs/src/lib.rs` - Module structure with fugle_version stub for csbindgen
- `cs/src/errors.rs` - Error code constants, error_to_code conversion, catch_panic helper
- `cs/src/types.rs` - Global tokio RUNTIME, string marshaling helpers, fugle_free_string export

## Decisions Made
- **csbindgen over UniFFI**: Phase 4 research showed csbindgen is .NET-specific and better suited than UniFFI's mobile-focused approach
- **Negative error codes**: SUCCESS = 0, errors use negative integers for C-style FFI conventions
- **Rate limit detection**: ApiError with status 429 maps to ERROR_RATE_LIMITED for special handling
- **Global runtime**: Lazy-initialized tokio runtime avoids per-call overhead for async operations

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - csbindgen integration was straightforward, error code mapping covered all MarketDataError variants without gaps.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

FFI foundation complete. Ready for:
- 04-02: REST API client implementation with async bridging
- 04-03: WebSocket client with EventHandler streaming
- 04-04: C# wrapper layer with Task-based async and exception hierarchy

All subsequent plans depend on this error handling and panic recovery infrastructure.

---
*Phase: 04-csharp-binding*
*Completed: 2026-01-31*
