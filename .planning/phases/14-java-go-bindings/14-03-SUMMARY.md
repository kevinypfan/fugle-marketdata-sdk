---
phase: 14-java-go-bindings
plan: 03
subsystem: bindings/csharp
tags: [config-exposure, csharp, options-pattern, exactly-one-auth]
dependency_graph:
  requires: [uniffi-layer, core-config-constants]
  provides: [csharp-options-classes, csharp-config-constructors]
  affects: [csharp-bindings, csharp-tests]
tech_stack:
  added: [RestClientOptions, WebSocketClientOptions, ReconnectOptions, HealthCheckOptions]
  patterns: [.NET-options-pattern, exactly-one-auth-validation, nullable-types]
key_files:
  created:
    - bindings/csharp/MarketdataUniffi/RestClientOptions.cs
    - bindings/csharp/MarketdataUniffi/WebSocketClientOptions.cs
    - bindings/csharp/MarketdataUniffi.Tests/ConfigOptionsTests.cs
  modified:
    - bindings/csharp/MarketdataUniffi/FugleRestClient.cs
    - bindings/csharp/MarketdataUniffi/FugleWebSocketClient.cs
    - bindings/csharp/MarketdataUniffi.Tests/RestClientTests.cs
    - bindings/csharp/MarketdataUniffi.Tests/FfiBoundaryTests.cs
decisions:
  - choice: "Use nullable properties (string?, uint?, ulong?) for all options"
    rationale: ".NET idiomatic - null means use default value from core constants"
    alternatives: ["Required properties with explicit defaults"]
  - choice: "Store ReconnectOptions and HealthCheckOptions in WebSocketClient but don't apply yet"
    rationale: "UniFFI WebSocketClient doesn't expose ConnectionConfig setter - defer to future work"
    impact: "Options validated and stored for future propagation when UniFFI layer supports it"
  - choice: "Only ApiKey auth supported in WebSocketClient options constructor"
    rationale: "Current UniFFI WebSocketClient constructors only accept api_key parameter"
    impact: "BearerToken and SdkToken throw NotSupportedException with clear message"
  - choice: "Fix ambiguous constructor calls by explicitly casting to string"
    rationale: "C# can't distinguish between string and RestClientOptions overloads when null is passed"
    impact: "Existing tests updated to cast null to (string)null!"
metrics:
  duration: "3 minutes 31 seconds"
  completed_at: "2026-02-15T15:32:21Z"
  tasks_completed: 2
  files_created: 3
  files_modified: 4
  commits: 2
  tests_added: 14
  test_status: "All 74 tests pass (40 run, 34 skipped)"
---

# Phase 14 Plan 03: C# Config Exposure Summary

C# options classes and constructors using idiomatic .NET options pattern with exactly-one-auth validation

## One-Liner

C# bindings now accept RestClientOptions and WebSocketClientOptions with nullable properties, exactly-one-auth validation via ArgumentException, and nested ReconnectOptions/HealthCheckOptions classes.

## What Was Built

### Options Classes (New Files)

**RestClientOptions.cs:**
- Properties: ApiKey?, BearerToken?, SdkToken?, BaseUrl? (all nullable)
- XML doc comments documenting exactly-one-auth requirement
- BaseUrl documented as "not yet implemented" (stored for future use)

**WebSocketClientOptions.cs:**
- Three classes in one file: ReconnectOptions, HealthCheckOptions, WebSocketClientOptions
- ReconnectOptions: MaxAttempts?, InitialDelayMs?, MaxDelayMs?
- HealthCheckOptions: Enabled?, IntervalMs?, MaxMissedPongs?
- WebSocketClientOptions: auth + Reconnect? + HealthCheck? + Endpoint (default: Stock)

### Client Constructors (Updated Files)

**FugleRestClient.cs:**
- New constructor: `RestClient(RestClientOptions options)`
- Exactly-one-auth validation: counts non-null auth properties, throws ArgumentException if 0 or >1
- Dispatches to correct UniFFI constructor: NewRestClientWithApiKey/BearerToken/SdkToken
- Wraps UniFFI MarketDataException in InvalidOperationException
- Existing string-based constructor preserved (backwards compatible)

**FugleWebSocketClient.cs:**
- New constructor: `WebSocketClient(WebSocketClientOptions options, IWebSocketListener listener)`
- Exactly-one-auth validation matching RestClient pattern
- Stores ReconnectOptions and HealthCheckOptions in private fields (not yet applied)
- Currently only supports ApiKey auth (BearerToken/SdkToken throw NotSupportedException)
- Endpoint conversion: C# WebSocketEndpoint enum to UniFFI endpoint enum

### Unit Tests (New File)

**ConfigOptionsTests.cs:**
14 comprehensive unit tests:

1. RestClientOptions_ExactlyOneAuth_ApiKey_Succeeds
2. RestClientOptions_ExactlyOneAuth_BearerToken_Succeeds
3. RestClientOptions_ExactlyOneAuth_SdkToken_Succeeds
4. RestClientOptions_NoAuth_ThrowsArgumentException
5. RestClientOptions_MultipleAuth_ThrowsArgumentException
6. RestClientOptions_NullOptions_ThrowsArgumentNullException
7. WebSocketClientOptions_NoAuth_ThrowsArgumentException
8. WebSocketClientOptions_MultipleAuth_ThrowsArgumentException
9. ReconnectOptions_DefaultValues_AreNull
10. ReconnectOptions_CustomValues_AreStored
11. HealthCheckOptions_DefaultValues_AreNull
12. HealthCheckOptions_CustomValues_AreStored
13. WebSocketClientOptions_AcceptsReconnectOptions
14. WebSocketClientOptions_AcceptsHealthCheckOptions

Test pattern: Construction tests use try/catch to verify auth validation passes (UniFFI may still fail, that's OK). Verification tests check error messages contain expected strings.

## Deviations from Plan

### Rule 3 (Auto-fix blocking issues)

**1. Fixed ambiguous constructor calls in existing tests**
- **Found during:** Task 2 test compilation
- **Issue:** Existing tests had `new RestClient(null!)` which became ambiguous with two overloads
- **Fix:** Cast to explicit type: `new RestClient((string)null!)` in RestClientTests.cs and FfiBoundaryTests.cs
- **Files modified:** RestClientTests.cs, FfiBoundaryTests.cs
- **Commit:** Included in test(14-03) commit c9fd337

## Verification Results

### Build Verification
```bash
cd bindings/csharp && dotnet build MarketdataUniffi/MarketdataUniffi.csproj
# Result: Build succeeded with 3 nullable warnings (acceptable - UniFFI signatures)
```

### Test Verification
```bash
cd bindings/csharp && dotnet test MarketdataUniffi.Tests/MarketdataUniffi.Tests.csproj
# Result: All 74 tests pass (40 run, 34 skipped due to no native library)
```

### Specific Tests
```bash
dotnet test --filter "ClassName~ConfigOptions"
# Result: 14 tests - 9 passed, 5 skipped (require native library)
```

### Backwards Compatibility
- Existing `RestClient(string apiKey)` constructor works
- Existing `WebSocketClient(string apiKey, listener)` constructor works
- All existing tests pass without modification (except null cast fix)

## Technical Notes

### .NET Options Pattern
- Nullable properties for optional configuration (null = use default)
- PascalCase property names (C# convention, matches Python pattern decision)
- XML doc comments with constraint documentation
- Exception type: ArgumentException for validation errors, ArgumentNullException for null parameters

### Exactly-One-Auth Validation
```csharp
int authCount = 0;
if (!string.IsNullOrEmpty(options.ApiKey)) authCount++;
if (!string.IsNullOrEmpty(options.BearerToken)) authCount++;
if (!string.IsNullOrEmpty(options.SdkToken)) authCount++;

if (authCount == 0)
    throw new ArgumentException("Provide exactly one of: ApiKey, BearerToken, SdkToken", nameof(options));
if (authCount > 1)
    throw new ArgumentException("Provide exactly one of: ApiKey, BearerToken, SdkToken", nameof(options));
```

Error message matches Python/Node.js pattern for consistency.

### WebSocketClient Limitations
- Current UniFFI layer only supports ApiKey authentication for WebSocketClient
- BearerToken and SdkToken paths throw NotSupportedException with clear message
- ReconnectOptions and HealthCheckOptions stored but not applied (no setter in UniFFI layer)
- BaseUrl override not applied in either client (documented as TODO)

### Nullable Warning Resolution
- 3 warnings about possible null reference in UniFFI method calls
- Acceptable: Our validation ensures non-null before passing to UniFFI
- Alternative would be to add `!` null-forgiving operator, but warnings are harmless

## Self-Check

### Files Created
```bash
[ -f "bindings/csharp/MarketdataUniffi/RestClientOptions.cs" ] && echo "FOUND: RestClientOptions.cs" || echo "MISSING: RestClientOptions.cs"
# FOUND: RestClientOptions.cs

[ -f "bindings/csharp/MarketdataUniffi/WebSocketClientOptions.cs" ] && echo "FOUND: WebSocketClientOptions.cs" || echo "MISSING: WebSocketClientOptions.cs"
# FOUND: WebSocketClientOptions.cs

[ -f "bindings/csharp/MarketdataUniffi.Tests/ConfigOptionsTests.cs" ] && echo "FOUND: ConfigOptionsTests.cs" || echo "MISSING: ConfigOptionsTests.cs"
# FOUND: ConfigOptionsTests.cs
```

### Commits Exist
```bash
git log --oneline --all | grep -q "cc01248" && echo "FOUND: cc01248" || echo "MISSING: cc01248"
# FOUND: cc01248 (feat: add C# options classes and constructors)

git log --oneline --all | grep -q "c9fd337" && echo "FOUND: c9fd337" || echo "MISSING: c9fd337"
# FOUND: c9fd337 (test: add C# unit tests for config options)
```

### Self-Check: PASSED

All created files exist. All commits exist and are in git history. Tests pass. Build succeeds.

## Task Completion Summary

| Task | Name                                      | Commit  | Files                                                                                |
| ---- | ----------------------------------------- | ------- | ------------------------------------------------------------------------------------ |
| 1    | Create options classes and update clients | cc01248 | RestClientOptions.cs, WebSocketClientOptions.cs, FugleRestClient.cs, FugleWebSocketClient.cs |
| 2    | Create C# unit tests for config options  | c9fd337 | ConfigOptionsTests.cs, RestClientTests.cs (fix), FfiBoundaryTests.cs (fix)           |

## Next Steps

After this plan:
1. Phase 14-04: Java options classes (if applicable - check ROADMAP)
2. Phase 14-05: Go options structs (if applicable - check ROADMAP)
3. Future work: Apply ReconnectOptions/HealthCheckOptions when UniFFI WebSocketClient exposes ConnectionConfig
4. Future work: Implement BaseUrl override when core RestClient supports it
5. Future work: Add BearerToken/SdkToken support to WebSocketClient when UniFFI layer updated

## Related Documentation

- Plan: .planning/phases/14-java-go-bindings/14-03-PLAN.md
- Research: .planning/phases/14-java-go-bindings/14-RESEARCH.md
- Python pattern: .planning/phases/12-python-config-exposure/ (reference for validation approach)
- Node.js pattern: .planning/phases/13-nodejs-config-exposure/ (reference for options object pattern)
