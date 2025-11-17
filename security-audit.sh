#!/usr/bin/env bash
#
# Security Audit Script for MyT2ABRP
#
# Performs comprehensive security checks on the codebase including:
# - Dependency vulnerability scanning
# - Secret detection
# - Code quality analysis
# - WASM binary analysis
# - Configuration validation
#
# Usage:
#   ./security-audit.sh [--fix]
#
# Options:
#   --fix    Attempt to automatically fix issues where possible

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

FIX_MODE=false
if [ "$1" = "--fix" ]; then
    FIX_MODE=true
fi

ISSUES_FOUND=0

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
}

log_error() {
    echo -e "${RED}✗${NC} $1"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
}

section() {
    echo ""
    echo -e "${BLUE}$1${NC}"
    echo "================================================================"
}

# Check if command exists
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_warn "$1 not found. Skipping $2 check."
        echo "    Install with: $3"
        return 1
    fi
    return 0
}

section "Security Audit for MyT2ABRP"
echo "Fix mode: $FIX_MODE"
echo ""

# 1. Rust Dependency Audit
section "1. Rust Dependency Vulnerability Scan"
if check_command "cargo-audit" "Rust dependencies" "cargo install cargo-audit"; then
    cd web-ui
    if cargo audit; then
        log_success "No known vulnerabilities in Rust dependencies"
    else
        log_error "Vulnerabilities found in Rust dependencies!"
        if [ "$FIX_MODE" = true ]; then
            log_info "Attempting to fix..."
            cargo update
        fi
    fi
    cd ..
fi

# 2. Secret Detection
section "2. Secret Detection"
log_info "Scanning for hardcoded secrets and sensitive data..."

# Check for common secret patterns
SECRETS_FOUND=0

# API keys, tokens, passwords
if grep -r -i "api[_-]key\|api[_-]secret\|password\|token\|secret" \
    --include="*.rs" \
    --include="*.ts" \
    --include="*.js" \
    --include="*.yaml" \
    --include="*.yml" \
    --include="*.toml" \
    . 2>/dev/null | grep -v -E "(test|example|demo|placeholder|TODO|FIXME)" | grep -v ".git"; then
    log_warn "Potential secrets found in code (review manually)"
    SECRETS_FOUND=$((SECRETS_FOUND + 1))
fi

# Check for .env files not in .gitignore
if [ -f ".env" ] && ! grep -q ".env" .gitignore 2>/dev/null; then
    log_error ".env file exists but not in .gitignore!"
    if [ "$FIX_MODE" = true ]; then
        echo ".env" >> .gitignore
        log_success "Added .env to .gitignore"
    fi
fi

# Check for exposed private keys
if find . -name "*.pem" -o -name "*.key" -o -name "*_rsa" 2>/dev/null | grep -v ".git"; then
    log_warn "Private key files found - ensure they're in .gitignore"
fi

if [ $SECRETS_FOUND -eq 0 ]; then
    log_success "No obvious secrets detected"
fi

# 3. Dependency License Check
section "3. License Compliance"
log_info "Checking dependency licenses..."
cd web-ui
if check_command "cargo-license" "license checking" "cargo install cargo-license"; then
    cargo license > /tmp/licenses.txt 2>&1 || true

    # Check for problematic licenses
    if grep -i -E "(GPL|AGPL|SSPL)" /tmp/licenses.txt; then
        log_warn "Potentially restrictive licenses found (review manually)"
    else
        log_success "No obviously problematic licenses found"
    fi
fi
cd ..

# 4. Code Quality - Clippy
section "4. Code Quality Analysis (Clippy)"
cd web-ui
if cargo clippy -- -D warnings 2>&1; then
    log_success "Clippy checks passed"
else
    log_warn "Clippy found issues"
    if [ "$FIX_MODE" = true ]; then
        log_info "Running clippy --fix..."
        cargo clippy --fix --allow-dirty --allow-staged
    fi
fi
cd ..

# 5. Outdated Dependencies
section "5. Outdated Dependencies"
if check_command "cargo-outdated" "outdated dependencies" "cargo install cargo-outdated"; then
    cd web-ui
    if cargo outdated; then
        log_info "Check output above for outdated dependencies"
    fi
    cd ..
fi

# 6. File Permissions
section "6. File Permissions"
log_info "Checking for overly permissive files..."

# Check for world-writable files
if find . -type f -perm -002 2>/dev/null | grep -v ".git"; then
    log_warn "World-writable files found"
    if [ "$FIX_MODE" = true ]; then
        find . -type f -perm -002 -exec chmod o-w {} \;
        log_success "Fixed file permissions"
    fi
else
    log_success "File permissions look good"
fi

# 7. Configuration Validation
section "7. Configuration Validation"

# Check spin.toml
if [ -f "spin.toml" ]; then
    log_info "Validating spin.toml..."

    # Check for plaintext secrets
    if grep -i "password\|secret\|token" spin.toml; then
        log_error "Potential plaintext secrets in spin.toml!"
    else
        log_success "No plaintext secrets in spin.toml"
    fi
fi

# Check .env.example exists
if [ ! -f ".env.example" ]; then
    log_warn ".env.example not found - should exist for documentation"
else
    log_success ".env.example exists"
fi

# 8. Docker Security
section "8. Docker Security"
if [ -f "Dockerfile" ]; then
    log_info "Checking Dockerfile..."

    # Check for running as root
    if ! grep -q "USER" Dockerfile; then
        log_warn "Dockerfile doesn't specify non-root USER"
    else
        log_success "Dockerfile specifies non-root user"
    fi

    # Check for COPY --chown
    if grep "COPY" Dockerfile | grep -v "COPY --chown"; then
        log_warn "COPY commands without --chown found"
    fi
fi

# 9. WASM Binary Analysis
section "9. WASM Binary Analysis"
WASM_FILE="web-ui/target/wasm32-wasip2/release/web_ui.wasm"
if [ -f "$WASM_FILE" ]; then
    SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
    SIZE_MB=$((SIZE / 1024 / 1024))

    log_info "WASM binary size: ${SIZE_MB}MB"

    if [ $SIZE -gt 10000000 ]; then
        log_warn "WASM binary is larger than 10MB - consider size optimization"
    else
        log_success "WASM binary size is reasonable"
    fi

    # Check for debug symbols
    if command -v wasm-objdump &> /dev/null; then
        if wasm-objdump -h "$WASM_FILE" | grep -q "debug"; then
            log_warn "Debug symbols found in release binary"
        else
            log_success "No debug symbols in release binary"
        fi
    fi
else
    log_info "WASM binary not found (run build first)"
fi

# 10. Git Security
section "10. Git Configuration"

# Check .gitignore
if [ -f ".gitignore" ]; then
    REQUIRED_IGNORES=(".env" "*.pem" "*.key" "target/" "node_modules/" ".spin/")

    for pattern in "${REQUIRED_IGNORES[@]}"; do
        if ! grep -q "$pattern" .gitignore; then
            log_warn "$pattern not in .gitignore"
            if [ "$FIX_MODE" = true ]; then
                echo "$pattern" >> .gitignore
                log_success "Added $pattern to .gitignore"
            fi
        fi
    done
else
    log_error ".gitignore not found!"
fi

# Summary
section "Security Audit Summary"
echo ""
if [ $ISSUES_FOUND -eq 0 ]; then
    log_success "Security audit complete - no critical issues found!"
    exit 0
else
    log_warn "Security audit found $ISSUES_FOUND issue(s)"
    if [ "$FIX_MODE" = false ]; then
        echo ""
        log_info "Run with --fix to attempt automatic fixes"
    fi
    exit 1
fi
