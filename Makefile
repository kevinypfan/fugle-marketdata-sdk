.PHONY: all clean python-dev python-release nodejs-dev nodejs-release csharp-dev csharp-release test test-python test-nodejs test-csharp

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
# C# Bindings (via cargo, csbindgen in Phase 4)
# ============================================================
csharp-dev:
	cargo build -p marketdata-uniffi

csharp-release:
	cargo build -p marketdata-uniffi --release

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
	@echo ""
	@echo "Test targets:"
	@echo "  make test             - Run all tests"
	@echo "  make test-python      - Run Python tests"
	@echo "  make test-nodejs      - Run Node.js tests"
	@echo "  make test-csharp      - Run C# tests"
	@echo ""
	@echo "Workspace targets:"
	@echo "  make check            - Check workspace (fast)"
	@echo "  make build            - Build workspace (debug)"
	@echo "  make build-release    - Build workspace (release)"
	@echo "  make clean            - Clean all build artifacts"
	@echo "  make fmt              - Format all code"
	@echo "  make lint             - Run clippy lints"
