# Notizia Development Makefile
# 
# This Makefile provides convenient targets for common development tasks,
# particularly around testing and code coverage.

.PHONY: help test coverage coverage-check coverage-lcov test-coverage clean-coverage install-tools

# Default target - show help
help:
	@echo "Notizia Development Commands"
	@echo ""
	@echo "Testing:"
	@echo "  make test              - Run all tests in the workspace"
	@echo "  make test-lib          - Run only library unit tests"
	@echo "  make test-integration  - Run only integration tests"
	@echo "  make test-doc          - Run only documentation tests"
	@echo ""
	@echo "Coverage:"
	@echo "  make coverage          - Generate HTML coverage report and open in browser"
	@echo "  make coverage-check    - Check if coverage meets 90% threshold"
	@echo "  make coverage-lcov     - Generate LCOV format coverage report"
	@echo "  make test-coverage     - Run tests with coverage tracking (terminal output)"
	@echo "  make clean-coverage    - Remove all coverage artifacts"
	@echo ""
	@echo "Development:"
	@echo "  make install-tools     - Install required development tools"
	@echo "  make fmt               - Format code with rustfmt"
	@echo "  make clippy            - Run clippy lints"
	@echo "  make build             - Build the workspace"
	@echo "  make clean             - Clean build artifacts"

# Testing targets
test:
	cargo test --workspace --verbose

test-lib:
	cargo test --workspace --lib

test-integration:
	cargo test --workspace --tests

test-doc:
	cargo test --workspace --doc

# Coverage targets
coverage:
	@echo "Generating HTML coverage report..."
	cargo llvm-cov --workspace --html
	@echo "Opening coverage report in browser..."
	@if [ "$$(uname)" = "Darwin" ]; then \
		open target/llvm-cov/html/index.html; \
	elif [ "$$(uname)" = "Linux" ]; then \
		xdg-open target/llvm-cov/html/index.html 2>/dev/null || echo "Please open target/llvm-cov/html/index.html in your browser"; \
	else \
		echo "Please open target/llvm-cov/html/index.html in your browser"; \
	fi

coverage-check:
	@echo "Checking coverage threshold (90%)..."
	@cargo llvm-cov --workspace --fail-under-lines 90 && \
		echo "✓ Coverage meets 90% threshold!" || \
		(echo "⚠️  Coverage is below 90% threshold" && exit 1)

coverage-lcov:
	@echo "Generating LCOV coverage report..."
	cargo llvm-cov --workspace --lcov --output-path lcov.info
	@echo "Coverage report generated: lcov.info"

test-coverage:
	@echo "Running tests with coverage..."
	cargo llvm-cov --workspace

clean-coverage:
	@echo "Cleaning coverage artifacts..."
	rm -rf target/llvm-cov/
	rm -f lcov.info
	@echo "Coverage artifacts removed"

# Development targets
install-tools:
	@echo "Installing development tools..."
	cargo install cargo-llvm-cov
	cargo install cargo-watch
	@echo "Tools installed successfully"

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

build:
	cargo build --workspace --verbose

clean:
	cargo clean
