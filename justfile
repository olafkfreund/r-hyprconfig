# Justfile for r-hyprconfig development
# https://github.com/casey/just

# Default recipe - show available commands
default:
    @just --list

# Development commands

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the application in debug mode
run *ARGS:
    cargo run -- {{ARGS}}

# Run the application with debug logging
debug *ARGS:
    RUST_LOG=debug cargo run -- --debug {{ARGS}}

# Run tests
test:
    cargo test --all-targets --all-features

# Run tests with output
test-verbose:
    cargo test --all-targets --all-features -- --nocapture

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all --check

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Fix clippy warnings automatically where possible
clippy-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Run security audit
audit:
    cargo audit

# Check for outdated dependencies
outdated:
    cargo outdated

# Clean build artifacts
clean:
    cargo clean

# Generate and open documentation
docs:
    cargo doc --open --no-deps

# Watch for changes and rebuild
watch:
    cargo watch -x check

# Watch for changes and run
watch-run:
    cargo watch -x 'run -- --debug'

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Lint everything (format, clippy, audit)
lint: fmt clippy audit

# Full CI check (lint + test)
ci: lint test

# Development setup
setup:
    @echo "🔧 Setting up development environment..."
    rustup component add rustfmt clippy rust-src
    cargo install cargo-watch cargo-audit cargo-outdated cargo-edit
    @echo "✅ Development environment ready!"

# Nix-specific commands

# Build with Nix
nix-build:
    nix build .#r-hyprconfig

# Run with Nix
nix-run *ARGS:
    nix run .#r-hyprconfig -- {{ARGS}}

# Enter development shell
nix-shell:
    nix develop

# Update flake inputs
nix-update:
    nix flake update

# Check flake
nix-check:
    nix flake check

# Development environment with devenv
devenv:
    devenv shell

# Run development environment with processes
devenv-up:
    devenv up

# Package management

# Add a new dependency
add-dep PACKAGE:
    cargo add {{PACKAGE}}

# Add a new development dependency
add-dev-dep PACKAGE:
    cargo add --dev {{PACKAGE}}

# Add a new build dependency
add-build-dep PACKAGE:
    cargo add --build {{PACKAGE}}

# Remove a dependency
rm-dep PACKAGE:
    cargo rm {{PACKAGE}}

# Update dependencies
update-deps:
    cargo update

# Release commands

# Create a new release (requires version)
release VERSION:
    @echo "🚀 Creating release {{VERSION}}..."
    git checkout main
    git pull origin main
    cargo set-version {{VERSION}}
    cargo check
    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version to {{VERSION}}"
    git tag -a "v{{VERSION}}" -m "Release {{VERSION}}"
    @echo "✅ Release {{VERSION}} created!"
    @echo "Run 'git push origin main && git push origin v{{VERSION}}' to publish"

# Benchmark (if benchmarks exist)
bench:
    cargo bench

# Example configurations for testing

# Create example config for testing
example-config:
    @echo "📄 Creating example configuration..."
    mkdir -p examples
    cp templates/default_hyprland.conf examples/test_hyprland.conf
    @echo "✅ Example config created at examples/test_hyprland.conf"

# Run with example config
test-config:
    @echo "🧪 Testing with example configuration..."
    cargo run -- --debug

# Docker commands (if needed)

# Build Docker image
docker-build:
    docker build -t r-hyprconfig .

# Run in Docker
docker-run:
    docker run -it --rm r-hyprconfig

# Development helpers

# Show project information
info:
    @echo "📊 Project Information"
    @echo "======================"
    @echo "Name: r-hyprconfig"
    @echo "Version: $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"
    @echo ""
    @echo "📁 Project structure:"
    @tree -I 'target|.git' -L 2

# Count lines of code
loc:
    @echo "📊 Lines of Code"
    @echo "================"
    find src -name "*.rs" -exec wc -l {} + | tail -1
    @echo ""
    @echo "📁 Breakdown by file:"
    find src -name "*.rs" -exec wc -l {} +

# Show TODOs and FIXMEs
todos:
    @echo "📝 TODOs and FIXMEs"
    @echo "==================="
    @rg -n "TODO|FIXME|XXX|HACK" src/ || echo "No TODOs found! 🎉"

# Profile the application (requires cargo-flamegraph)
profile:
    cargo flamegraph --bin r-hyprconfig

# Memory check with valgrind
memcheck:
    cargo build
    valgrind --tool=memcheck --leak-check=full ./target/debug/r-hyprconfig --help