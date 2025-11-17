#!/usr/bin/env bash
#
# MyT2ABRP Development Helper Script
#
# Provides convenient commands for common development tasks.
#
# Usage:
#   ./dev.sh [command]
#
# Commands:
#   build       - Build all components
#   run         - Build and run the application
#   test        - Run all tests
#   format      - Format all code
#   lint        - Run linters
#   clean       - Clean build artifacts
#   watch       - Watch for changes and rebuild (requires cargo-watch)
#   docs        - Generate and open documentation
#   help        - Show this help message

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Command implementations
cmd_build() {
    log_info "Building web-ui component..."
    cd web-ui
    cargo build --target wasm32-wasip2 --release
    cd ..

    log_info "Building with Spin..."
    spin build

    log_success "Build complete!"
}

cmd_run() {
    log_info "Building and starting application..."
    cmd_build

    log_info "Starting Spin server on http://localhost:3000"
    log_warn "Press Ctrl+C to stop"
    spin up
}

cmd_test() {
    log_info "Running Rust tests..."
    cd web-ui
    cargo test
    cd ..

    if [ -d "tests" ] && [ -f "tests/package.json" ]; then
        log_info "Running E2E tests..."
        cd tests
        if [ -d "node_modules" ]; then
            npm test
        else
            log_warn "Node modules not installed. Run: cd tests && npm install"
        fi
        cd ..
    else
        log_warn "E2E tests directory not found or not configured"
    fi

    log_success "Tests complete!"
}

cmd_format() {
    log_info "Formatting Rust code..."
    cd web-ui
    cargo fmt
    cd ..

    if [ -d "tests" ]; then
        log_info "Formatting TypeScript code..."
        cd tests
        if [ -d "node_modules" ]; then
            npx prettier --write "**/*.{ts,js,json}"
        else
            log_warn "Node modules not installed. Skipping TypeScript formatting."
        fi
        cd ..
    fi

    log_success "Formatting complete!"
}

cmd_lint() {
    log_info "Running Clippy on Rust code..."
    cd web-ui
    cargo clippy -- -D warnings
    cd ..

    log_info "Checking Rust formatting..."
    cd web-ui
    cargo fmt -- --check
    cd ..

    log_success "Linting complete!"
}

cmd_clean() {
    log_info "Cleaning build artifacts..."

    if [ -d "web-ui/target" ]; then
        rm -rf web-ui/target
        log_success "Removed web-ui/target"
    fi

    if [ -d ".spin" ]; then
        rm -rf .spin
        log_success "Removed .spin cache"
    fi

    if [ -d "tests/test-results" ]; then
        rm -rf tests/test-results
        log_success "Removed test results"
    fi

    if [ -d "tests/playwright-report" ]; then
        rm -rf tests/playwright-report
        log_success "Removed playwright reports"
    fi

    log_success "Clean complete!"
}

cmd_watch() {
    log_info "Starting watch mode..."

    if ! command -v cargo-watch &> /dev/null; then
        log_error "cargo-watch not found. Install with: cargo install cargo-watch"
        exit 1
    fi

    log_warn "Watching for changes. Press Ctrl+C to stop."
    cd web-ui
    cargo watch -x 'build --target wasm32-wasip2 --release' -s 'cd .. && spin up'
}

cmd_docs() {
    log_info "Generating documentation..."
    cd web-ui
    cargo doc --no-deps --target wasm32-wasip2
    cd ..

    log_success "Documentation generated!"
    log_info "Opening documentation in browser..."

    # Try to open docs in browser
    if command -v open &> /dev/null; then
        open web-ui/target/wasm32-wasip2/doc/web_ui/index.html
    elif command -v xdg-open &> /dev/null; then
        xdg-open web-ui/target/wasm32-wasip2/doc/web_ui/index.html
    else
        log_warn "Could not open browser automatically"
        echo "    Open: web-ui/target/wasm32-wasip2/doc/web_ui/index.html"
    fi
}

cmd_help() {
    cat << EOF
MyT2ABRP Development Helper Script

Usage:
    ./dev.sh [command]

Commands:
    build       Build all components
    run         Build and run the application
    test        Run all tests (Rust + E2E)
    format      Format all code (Rust + TypeScript)
    lint        Run linters (Clippy + rustfmt check)
    clean       Clean build artifacts
    watch       Watch for changes and rebuild (requires cargo-watch)
    docs        Generate and open Rust documentation
    help        Show this help message

Examples:
    ./dev.sh build          # Build the project
    ./dev.sh run            # Build and start server
    ./dev.sh test           # Run all tests
    ./dev.sh format         # Format code
    ./dev.sh lint           # Check code quality
    ./dev.sh clean          # Clean build artifacts
    ./dev.sh watch          # Watch mode (auto-rebuild)
    ./dev.sh docs           # Generate docs

For more information, see README.md
EOF
}

# Main command dispatcher
main() {
    case "${1:-help}" in
        build)
            cmd_build
            ;;
        run)
            cmd_run
            ;;
        test)
            cmd_test
            ;;
        format)
            cmd_format
            ;;
        lint)
            cmd_lint
            ;;
        clean)
            cmd_clean
            ;;
        watch)
            cmd_watch
            ;;
        docs)
            cmd_docs
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            log_error "Unknown command: $1"
            echo ""
            cmd_help
            exit 1
            ;;
    esac
}

main "$@"
