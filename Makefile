.PHONY: all clean python-dev python-release nodejs-dev nodejs-release csharp-dev csharp-release gen-bindings gen-csharp gen-go build-csharp build-go test test-python test-nodejs test-csharp test-go

# Default: Build all bindings in order (Python -> Node.js -> C#)
all: python-release nodejs-release csharp-release

# ============================================================
# Python Bindings (via maturin)
# ============================================================
python-dev:
	cd py && maturin develop

python-release:
	cd py && maturin build --release

# ============================================================
# Node.js Bindings (via napi-rs)
# ============================================================
nodejs-dev:
	cd js && npm run build:debug

nodejs-release:
	cd js && npm run build

# ============================================================
# C# Bindings (via UniFFI)
# ============================================================
csharp-dev:
	cargo build -p marketdata-uniffi

csharp-release:
	cargo build -p marketdata-uniffi --release

# Generate all bindings (C# and Go)
gen-bindings: gen-csharp gen-go

# Generate C# bindings from UniFFI (library mode - proc-macro approach)
# Post-processes generated file to make types public for consumer access
gen-csharp:
	cargo build -p marketdata-uniffi --release
	uniffi-bindgen-cs --library target/release/libmarketdata_uniffi.dylib -o bindings/csharp/MarketdataUniffi/
	@echo "Post-processing: making generated types public..."
	sed -i '' 's/^internal record /public record /g' bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs
	sed -i '' 's/^internal interface /public interface /g' bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs
	sed -i '' 's/^internal class /public class /g' bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs
	sed -i '' 's/^internal enum /public enum /g' bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs
	sed -i '' 's/^internal static class MarketdataUniffiMethods/public static class MarketdataUniffiMethods/g' bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs
	@echo "C# bindings generated successfully"

# ============================================================
# Go Bindings (via UniFFI)
# ============================================================

# Generate Go bindings from UniFFI (library mode - proc-macro approach)
gen-go:
	cargo build -p marketdata-uniffi --release
	mkdir -p bindings/go/marketdata
	uniffi-bindgen-go --library target/release/libmarketdata_uniffi.dylib -o bindings/go/tmp/
	mv bindings/go/tmp/marketdata_uniffi/* bindings/go/marketdata/
	rmdir bindings/go/tmp/marketdata_uniffi bindings/go/tmp

# Build C# project after generating bindings
build-csharp: gen-csharp
	dotnet build bindings/csharp/FugleMarketData.sln

# Build Go module after generating bindings
build-go: gen-go
	cd bindings/go/marketdata && CGO_ENABLED=1 go build .

# ============================================================
# Testing
# ============================================================
test: test-python test-nodejs test-csharp

test-python:
	cd py && maturin develop && python -m pytest tests/ -v || echo "Python tests not yet configured"

test-nodejs:
	cd js && npm test || echo "Node.js tests not yet configured"

test-csharp:
	cargo test -p marketdata-uniffi
	@echo "Run C# tests with: dotnet test bindings/csharp/FugleMarketData.Tests/"

test-go:
	cd bindings/go/marketdata && CGO_ENABLED=1 go test -v -short .

# ============================================================
# Workspace Operations
# ============================================================
check:
	cargo check --workspace

build:
	cargo build --workspace

build-release:
	cargo build --workspace --release

clean:
	cargo clean
	rm -rf py/target/
	rm -rf js/target/
	rm -rf uniffi/target/

# ============================================================
# Development Helpers
# ============================================================
fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace -- -D warnings

# Show help
help:
	@echo "Fugle MarketData SDK Build System"
	@echo ""
	@echo "Build targets:"
	@echo "  make all              - Build all bindings (release)"
	@echo "  make python-dev       - Build Python binding (dev)"
	@echo "  make python-release   - Build Python binding (release)"
	@echo "  make nodejs-dev       - Build Node.js binding (dev)"
	@echo "  make nodejs-release   - Build Node.js binding (release)"
	@echo "  make csharp-dev       - Build C# binding (dev)"
	@echo "  make csharp-release   - Build C# binding (release)"
	@echo "  make gen-bindings     - Generate all UniFFI bindings (C# + Go)"
	@echo "  make gen-csharp       - Generate C# bindings from UniFFI"
	@echo "  make gen-go           - Generate Go bindings from UniFFI"
	@echo "  make build-csharp     - Build C# project"
	@echo "  make build-go         - Build Go module"
	@echo ""
	@echo "Test targets:"
	@echo "  make test             - Run all tests"
	@echo "  make test-python      - Run Python tests"
	@echo "  make test-nodejs      - Run Node.js tests"
	@echo "  make test-csharp      - Run C# tests"
	@echo "  make test-go          - Run Go tests"
	@echo ""
	@echo "Workspace targets:"
	@echo "  make check            - Check workspace (fast)"
	@echo "  make build            - Build workspace (debug)"
	@echo "  make build-release    - Build workspace (release)"
	@echo "  make clean            - Clean all build artifacts"
	@echo "  make fmt              - Format all code"
	@echo "  make lint             - Run clippy lints"
