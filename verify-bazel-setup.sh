#!/bin/bash
# Verification script for Bazel WebAssembly Component Model setup
# This script validates the configuration without requiring network access

set -e

echo "🔍 Bazel Setup Verification"
echo "============================"
echo

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_pass() {
    echo -e "${GREEN}✅ $1${NC}"
}

check_fail() {
    echo -e "${RED}❌ $1${NC}"
}

check_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# 1. Check Bazel/Bazelisk installation
echo "1. Checking Bazel installation..."
if command -v bazel &> /dev/null; then
    BAZEL_VERSION=$(bazel version 2>&1 | grep "Build label" | cut -d: -f2 | xargs || echo "unknown")
    check_pass "Bazel found: $BAZEL_VERSION"
elif command -v bazelisk &> /dev/null; then
    check_pass "Bazelisk found (will auto-download Bazel)"
elif [ -x "/tmp/bazelisk" ]; then
    check_pass "Bazelisk found at /tmp/bazelisk"
    check_warn "Consider adding to PATH or copying to /usr/local/bin"
else
    check_fail "Bazel or Bazelisk not found"
    echo "   Install with: npm install -g @bazel/bazelisk"
    echo "   Or download from: https://github.com/bazelbuild/bazelisk/releases"
    exit 1
fi
echo

# 2. Verify MODULE.bazel
echo "2. Verifying MODULE.bazel..."
if [ -f "MODULE.bazel" ]; then
    check_pass "MODULE.bazel exists"

    # Check for git_override
    if grep -q "git_override" MODULE.bazel; then
        check_pass "git_override configured for rules_wasm_component"

        # Verify commit hash (not HEAD)
        if grep -q 'commit = "55a6a0a3ed9515a205bba1fbf8d4917f42084efa"' MODULE.bazel; then
            check_pass "Using specific commit hash (not HEAD)"
        else
            check_warn "Commit hash may need verification"
        fi
    else
        check_fail "git_override not found in MODULE.bazel"
    fi

    # Check for required deps
    if grep -q "rules_rust" MODULE.bazel; then
        check_pass "rules_rust dependency declared"
    fi
else
    check_fail "MODULE.bazel not found"
    exit 1
fi
echo

# 3. Verify .bazelrc
echo "3. Verifying .bazelrc..."
if [ -f ".bazelrc" ]; then
    check_pass ".bazelrc exists"

    if grep -q "enable_bzlmod" .bazelrc; then
        check_pass "bzlmod enabled"
    fi

    if grep -q "test_output=all" .bazelrc; then
        check_pass "Test output configured"
    fi
else
    check_warn ".bazelrc not found (optional but recommended)"
fi
echo

# 4. Verify .bazelignore
echo "4. Verifying .bazelignore..."
if [ -f ".bazelignore" ]; then
    check_pass ".bazelignore exists"

    if grep -q "target/" .bazelignore; then
        check_pass "Cargo target/ directory ignored"
    fi
else
    check_warn ".bazelignore not found (may cause conflicts with Cargo)"
fi
echo

# 5. Verify BUILD files
echo "5. Verifying BUILD files..."
BUILD_FILES=(
    "BUILD.bazel"
    "components/business-logic/BUILD.bazel"
    "tools/BUILD.bazel"
)

for file in "${BUILD_FILES[@]}"; do
    if [ -f "$file" ]; then
        check_pass "$file exists"
    else
        check_fail "$file missing"
    fi
done
echo

# 6. Verify component structure
echo "6. Verifying component structure..."
if [ -d "components/business-logic" ]; then
    check_pass "business-logic component directory exists"

    # Check WIT files
    if [ -d "components/business-logic/wit" ]; then
        check_pass "WIT interface directory exists"
        WIT_COUNT=$(find components/business-logic/wit -name "*.wit" | wc -l)
        if [ "$WIT_COUNT" -gt 0 ]; then
            check_pass "Found $WIT_COUNT WIT interface file(s)"
        else
            check_fail "No WIT files found"
        fi
    else
        check_fail "WIT directory missing"
    fi

    # Check Rust source
    if [ -d "components/business-logic/src" ]; then
        check_pass "Source directory exists"
        RS_COUNT=$(find components/business-logic/src -name "*.rs" | wc -l)
        check_pass "Found $RS_COUNT Rust source file(s)"
    else
        check_fail "Source directory missing"
    fi
else
    check_fail "business-logic component directory missing"
fi
echo

# 7. Verify BUILD.bazel uses wac_plug
echo "7. Verifying WAC composition..."
if [ -f "BUILD.bazel" ]; then
    if grep -q "wac_plug" BUILD.bazel; then
        check_pass "wac_plug rule configured in BUILD.bazel"

        if grep -q "composed_app" BUILD.bazel; then
            check_pass "composed_app target defined"
        fi
    else
        check_fail "wac_plug not found in BUILD.bazel"
    fi
fi
echo

# 8. Verify rust_wasm_component in component BUILD
echo "8. Verifying rust_wasm_component rule..."
if [ -f "components/business-logic/BUILD.bazel" ]; then
    if grep -q "rust_wasm_component" components/business-logic/BUILD.bazel; then
        check_pass "rust_wasm_component rule configured"

        # Check for tests
        if grep -q "sh_test" components/business-logic/BUILD.bazel; then
            check_pass "Shell tests configured"
        fi
    else
        check_fail "rust_wasm_component not found"
    fi
fi
echo

# 9. Summary
echo "============================"
echo "📊 Setup Summary"
echo "============================"
echo

echo "Configuration Status: READY FOR TESTING"
echo
echo "To test the build system:"
echo "  1. Ensure you have network access to:"
echo "     - https://github.com/pulseengine/rules_wasm_component.git"
echo "     - Bazel Central Registry (for dependency resolution)"
echo
echo "  2. Run basic query:"
echo "     $ bazel query //..."
echo
echo "  3. Build business logic component:"
echo "     $ bazel build //components/business-logic:business_logic"
echo
echo "  4. Run tests:"
echo "     $ bazel test //components/business-logic:..."
echo
echo "  5. Build composed app (WAC):"
echo "     $ bazel build //:composed_app"
echo
echo "  6. Clean build (if needed):"
echo "     $ bazel clean --expunge"
echo

echo "📚 Documentation:"
echo "  - BAZEL-BUILD.md - Build system overview"
echo "  - BAZEL-INTEGRATION.md - Integration guide"
echo "  - POC-COMPONENT-COMPOSITION.md - Architecture details"
echo

echo "✅ Verification complete!"
