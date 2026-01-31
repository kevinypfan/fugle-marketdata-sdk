---
quick: 002
type: execute
autonomous: true
files_modified:
  - bindings/csharp/README.md
  - bindings/java/README.md
  - bindings/go/README.md

must_haves:
  truths:
    - "Each UniFFI binding (C#, Java, Go) has a README with installation and usage"
    - "READMEs follow consistent structure across all languages"
    - "Examples cover REST API and WebSocket for each language"
  artifacts:
    - path: "bindings/csharp/README.md"
      provides: "C# binding documentation"
    - path: "bindings/java/README.md"
      provides: "Java binding documentation"
    - path: "bindings/go/README.md"
      provides: "Go binding documentation"
---

<objective>
Add comprehensive README documentation for the three UniFFI language bindings (C#, Java, Go).

Purpose: Enable developers to quickly start using the SDK in their preferred language with clear installation instructions, usage examples, and API reference.

Output: Three README.md files following the structure established by py/README.md and js/README.md.
</objective>

<context>
@.planning/STATE.md
@py/README.md (reference structure)
@js/README.md (reference structure)
@uniffi/README.md (UniFFI overview)
@bindings/csharp/TestRestApi/Program.cs (C# usage patterns)
@bindings/java/examples/RestExample.java (Java usage patterns)
@bindings/go/examples/rest_example.go (Go usage patterns)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create C# README</name>
  <files>bindings/csharp/README.md</files>
  <action>
Create comprehensive README for C# binding following the structure from py/README.md:

1. **Header**: `# FugleMarketData.NET` - C# bindings for Fugle Market Data API

2. **Installation**:
   - Building from source (requires UniFFI native library)
   - Reference the .csproj in your project
   - Note: Requires .NET 8.0+

3. **Quick Start**:
   - REST API example (get stock quote for 2330)
   - WebSocket streaming example with callbacks

4. **Authentication**:
   - API Key: `new RestClient(apiKey)`
   - Bearer Token: `RestClient.WithBearerToken(token)`
   - SDK Token: `RestClient.WithSdkToken(token)`

5. **API Reference**:
   - RestClient methods (Stock.Intraday.*, FutOpt.Intraday.*)
   - WebSocketClient (Stock, FutOpt properties)
   - Event types and channels

6. **Error Handling**:
   - Try-catch patterns with FugleException
   - Error code table (same as other bindings)

7. **Full Examples**:
   - Complete REST example
   - Complete WebSocket example with async

Extract patterns from bindings/csharp/TestRestApi/Program.cs and TestWebSocket/Program.cs.
Use async/await patterns that are idiomatic for C#.
  </action>
  <verify>File exists and contains sections: Installation, Quick Start, Authentication, API Reference, Error Handling, Examples</verify>
  <done>bindings/csharp/README.md exists with comprehensive C# documentation</done>
</task>

<task type="auto">
  <name>Task 2: Create Java README</name>
  <files>bindings/java/README.md</files>
  <action>
Create comprehensive README for Java binding following the structure from py/README.md:

1. **Header**: `# FugleMarketData Java` - Java bindings for Fugle Market Data API

2. **Installation**:
   - Building with Gradle
   - Requires UniFFI native library
   - Setting up JNA library path
   - Note: Requires Java 21+

3. **Quick Start**:
   - REST API example (get stock quote)
   - WebSocket streaming example

4. **Authentication**:
   - Builder pattern: `FugleRestClient.builder().apiKey(key).build()`
   - Bearer token and SDK token alternatives

5. **API Reference**:
   - FugleRestClient methods
   - FugleWebSocketClient
   - Generated types (Quote, Ticker, etc.)
   - Async methods with CompletableFuture

6. **Error Handling**:
   - FugleException handling
   - Error code table

7. **Full Examples**:
   - Complete REST example
   - Complete WebSocket example

Extract patterns from bindings/java/examples/RestExample.java and WebSocketExample.java.
Use builder patterns and CompletableFuture that are idiomatic for Java.
  </action>
  <verify>File exists and contains sections: Installation, Quick Start, Authentication, API Reference, Error Handling, Examples</verify>
  <done>bindings/java/README.md exists with comprehensive Java documentation</done>
</task>

<task type="auto">
  <name>Task 3: Create Go README</name>
  <files>bindings/go/README.md</files>
  <action>
Create comprehensive README for Go binding following the structure from py/README.md:

1. **Header**: `# fugle-marketdata-go` - Go bindings for Fugle Market Data API

2. **Installation**:
   - `go get github.com/fugle-dev/fugle-marketdata-go`
   - Building native library requirement
   - CGO requirements

3. **Quick Start**:
   - REST API example (get stock quote)
   - WebSocket streaming example

4. **Authentication**:
   - `mkt.NewRestClientWithApiKey(apiKey)`
   - Bearer token and SDK token alternatives

5. **API Reference**:
   - Client methods
   - Typed response structs (Quote, Ticker, etc.)
   - Error handling with Go idioms

6. **Error Handling**:
   - Go-style error checking patterns
   - MarketDataException types
   - Error code table

7. **Full Examples**:
   - Complete REST example
   - Complete WebSocket example

Extract patterns from bindings/go/examples/rest_example.go.
Use pointer receivers and error handling that are idiomatic for Go.
  </action>
  <verify>File exists and contains sections: Installation, Quick Start, Authentication, API Reference, Error Handling, Examples</verify>
  <done>bindings/go/README.md exists with comprehensive Go documentation</done>
</task>

</tasks>

<verification>
- [ ] bindings/csharp/README.md exists with all sections
- [ ] bindings/java/README.md exists with all sections
- [ ] bindings/go/README.md exists with all sections
- [ ] Each README follows consistent structure
- [ ] Examples are complete and runnable
- [ ] Error codes are consistent across all READMEs
</verification>

<success_criteria>
All three UniFFI language bindings have comprehensive README documentation that enables developers to:
1. Install and set up the binding
2. Authenticate with the API
3. Make REST API calls
4. Use WebSocket streaming
5. Handle errors appropriately
</success_criteria>

<output>
After completion, verify all three README files exist and are consistent.
</output>
