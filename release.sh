#!/usr/bin/env bash
#
# Release Automation Script for MyT2ABRP
#
# Automates the release process including:
# - Version bumping
# - Changelog generation
# - Git tagging
# - Building release artifacts
# - Creating GitHub release
#
# Usage:
#   ./release.sh [version] [--dry-run]
#
# Examples:
#   ./release.sh 1.0.0          # Release v1.0.0
#   ./release.sh 1.1.0 --dry-run  # Preview what would be released

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Options
DRY_RUN=false

# Parse arguments
VERSION="$1"
if [ "$2" = "--dry-run" ]; then
    DRY_RUN=true
fi

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
    exit 1
}

section() {
    echo ""
    echo -e "${BLUE}$1${NC}"
    echo "================================================================"
}

# Validate version format
validate_version() {
    if [ -z "$VERSION" ]; then
        log_error "Version required. Usage: ./release.sh <version>"
    fi

    # Check semantic versioning format
    if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        log_error "Invalid version format. Use semantic versioning (e.g., 1.0.0, 1.0.0-beta.1)"
    fi

    log_success "Version format valid: $VERSION"
}

# Check prerequisites
check_prerequisites() {
    section "Checking Prerequisites"

    # Check git
    if ! command -v git &> /dev/null; then
        log_error "Git not found"
    fi
    log_success "Git installed"

    # Check if in git repo
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not in a git repository"
    fi

    # Check for uncommitted changes
    if [ -n "$(git status --porcelain)" ]; then
        log_error "Uncommitted changes detected. Commit or stash them first."
    fi
    log_success "No uncommitted changes"

    # Check if main/master branch
    current_branch=$(git branch --show-current)
    if [[ "$current_branch" != "main" && "$current_branch" != "master" ]]; then
        log_warn "Not on main/master branch (current: $current_branch)"
        if [ "$DRY_RUN" = false ]; then
            read -p "Continue anyway? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_error "Release cancelled"
            fi
        fi
    fi

    # Check if version tag already exists
    if git tag | grep -q "^v$VERSION$"; then
        log_error "Tag v$VERSION already exists"
    fi
    log_success "Tag v$VERSION is available"

    # Check required tools
    for tool in spin cargo node docker; do
        if command -v $tool &> /dev/null; then
            log_success "$tool installed"
        else
            log_warn "$tool not found (optional)"
        fi
    done
}

# Run tests
run_tests() {
    section "Running Tests"

    log_info "Running security audit..."
    if ./security-audit.sh > /dev/null 2>&1; then
        log_success "Security audit passed"
    else
        log_warn "Security audit found issues"
        if [ "$DRY_RUN" = false ]; then
            read -p "Continue anyway? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_error "Release cancelled"
            fi
        fi
    fi

    log_info "Running linters..."
    if ./dev.sh lint > /dev/null 2>&1; then
        log_success "Linters passed"
    else
        log_error "Linters failed! Fix issues before releasing."
    fi

    log_info "Running tests..."
    if ./dev.sh test > /dev/null 2>&1; then
        log_success "Tests passed"
    else
        log_error "Tests failed! Fix issues before releasing."
    fi
}

# Build release artifacts
build_artifacts() {
    section "Building Release Artifacts"

    log_info "Building web-ui component..."
    cd web-ui
    cargo build --target wasm32-wasip2 --release
    cd ..

    log_info "Building with Spin..."
    spin build

    # Check WASM size
    WASM_FILE="web-ui/target/wasm32-wasip2/release/web_ui.wasm"
    if [ -f "$WASM_FILE" ]; then
        SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
        SIZE_MB=$((SIZE / 1024 / 1024))
        log_info "WASM binary size: ${SIZE_MB}MB"

        if [ $SIZE -gt 10000000 ]; then
            log_warn "WASM binary is larger than 10MB"
        fi
    fi

    # Create release directory
    mkdir -p release/v$VERSION

    # Copy artifacts
    log_info "Copying artifacts..."
    cp "$WASM_FILE" "release/v$VERSION/"
    cp spin.toml "release/v$VERSION/"
    cp README.md "release/v$VERSION/"
    cp LICENSE "release/v$VERSION/" 2>/dev/null || true

    # Create tarball
    log_info "Creating release tarball..."
    tar czf "release/myt2abrp-v$VERSION.tar.gz" -C release "v$VERSION"

    log_success "Release artifacts created in release/"
}

# Generate changelog
generate_changelog() {
    section "Generating Changelog"

    # Get previous tag
    PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

    if [ -z "$PREV_TAG" ]; then
        log_info "No previous tag found. Generating full changelog..."
        RANGE="HEAD"
    else
        log_info "Generating changelog since $PREV_TAG..."
        RANGE="$PREV_TAG..HEAD"
    fi

    # Generate changelog
    CHANGELOG_FILE="release/v$VERSION/CHANGELOG.md"
    mkdir -p "release/v$VERSION"

    cat > "$CHANGELOG_FILE" << EOF
# MyT2ABRP v$VERSION

**Release Date**: $(date +%Y-%m-%d)

## What's New

EOF

    # Parse commits
    git log $RANGE --pretty=format:"%s (%h)" --no-merges | while read line; do
        # Categorize commits
        if [[ "$line" == feat:* ]] || [[ "$line" == "feat("* ]]; then
            echo "### ✨ Features" >> "$CHANGELOG_FILE.tmp"
            echo "- ${line#feat*: }" >> "$CHANGELOG_FILE.tmp"
        elif [[ "$line" == fix:* ]] || [[ "$line" == "fix("* ]]; then
            echo "### 🐛 Bug Fixes" >> "$CHANGELOG_FILE.tmp"
            echo "- ${line#fix*: }" >> "$CHANGELOG_FILE.tmp"
        elif [[ "$line" == docs:* ]]; then
            echo "### 📝 Documentation" >> "$CHANGELOG_FILE.tmp"
            echo "- ${line#docs: }" >> "$CHANGELOG_FILE.tmp"
        elif [[ "$line" == perf:* ]]; then
            echo "### ⚡ Performance" >> "$CHANGELOG_FILE.tmp"
            echo "- ${line#perf: }" >> "$CHANGELOG_FILE.tmp"
        fi
    done

    # Sort and deduplicate sections
    if [ -f "$CHANGELOG_FILE.tmp" ]; then
        sort -u "$CHANGELOG_FILE.tmp" >> "$CHANGELOG_FILE"
        rm "$CHANGELOG_FILE.tmp"
    fi

    # Add statistics
    COMMIT_COUNT=$(git rev-list $RANGE --count)
    CONTRIBUTOR_COUNT=$(git shortlog -s $RANGE | wc -l)

    cat >> "$CHANGELOG_FILE" << EOF

## Statistics

- **Commits**: $COMMIT_COUNT
- **Contributors**: $CONTRIBUTOR_COUNT

## Installation

\`\`\`bash
# Download release
wget https://github.com/avrabe/spin_myT2ABRP/releases/download/v$VERSION/myt2abrp-v$VERSION.tar.gz

# Extract
tar xzf myt2abrp-v$VERSION.tar.gz

# Or deploy with Spin
spin deploy --from ghcr.io/avrabe/spin_myt2abrp:v$VERSION
\`\`\`

## Docker

\`\`\`bash
docker pull ghcr.io/avrabe/spin_myt2abrp:v$VERSION
docker run -p 3000:3000 ghcr.io/avrabe/spin_myt2abrp:v$VERSION
\`\`\`

---

**Full Changelog**: https://github.com/avrabe/spin_myT2ABRP/compare/$PREV_TAG...v$VERSION
EOF

    log_success "Changelog generated: $CHANGELOG_FILE"
}

# Create git tag
create_tag() {
    section "Creating Git Tag"

    TAG_MESSAGE="Release v$VERSION"

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would create tag v$VERSION with message: $TAG_MESSAGE"
        return
    fi

    log_info "Creating tag v$VERSION..."
    git tag -a "v$VERSION" -m "$TAG_MESSAGE"

    log_info "Pushing tag to remote..."
    git push origin "v$VERSION"

    log_success "Tag v$VERSION created and pushed"
}

# Create GitHub release
create_github_release() {
    section "Creating GitHub Release"

    if ! command -v gh &> /dev/null; then
        log_warn "GitHub CLI (gh) not found. Skipping GitHub release creation."
        log_info "Install gh from: https://cli.github.com/"
        log_info "Then run: gh release create v$VERSION"
        return
    fi

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would create GitHub release with artifacts"
        return
    fi

    log_info "Creating GitHub release..."

    CHANGELOG_FILE="release/v$VERSION/CHANGELOG.md"
    ARTIFACT="release/myt2abrp-v$VERSION.tar.gz"

    gh release create "v$VERSION" \
        --title "MyT2ABRP v$VERSION" \
        --notes-file "$CHANGELOG_FILE" \
        "$ARTIFACT"

    log_success "GitHub release created: https://github.com/avrabe/spin_myT2ABRP/releases/tag/v$VERSION"
}

# Main release process
main() {
    cat << EOF
$(echo -e "${BLUE}")
╔═══════════════════════════════════════════════════╗
║           MyT2ABRP Release Automation             ║
╚═══════════════════════════════════════════════════╝
$(echo -e "${NC}")
EOF

    validate_version
    check_prerequisites
    run_tests
    build_artifacts
    generate_changelog
    create_tag
    create_github_release

    section "Release Complete"

    if [ "$DRY_RUN" = true ]; then
        log_warn "DRY RUN - No changes were made"
        log_info "Review release/ directory for generated artifacts"
    else
        log_success "Release v$VERSION completed successfully!"
        echo ""
        log_info "Next steps:"
        echo "  1. Announce release (blog, social media, etc.)"
        echo "  2. Update documentation if needed"
        echo "  3. Deploy to production if applicable"
        echo ""
        log_info "Release artifacts:"
        echo "  - GitHub: https://github.com/avrabe/spin_myT2ABRP/releases/tag/v$VERSION"
        echo "  - Local: release/myt2abrp-v$VERSION.tar.gz"
    fi
}

main
