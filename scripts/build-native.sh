#!/bin/bash
# Build all components natively for testing
# Usage: ./scripts/build-native.sh [--release]

set -e

PROFILE="dev"
if [ "$1" == "--release" ]; then
    PROFILE="release"
    RELEASE_FLAG="--release"
fi

echo "🏗️  Building all components natively ($PROFILE)"
echo

START_TIME=$(date +%s)

# Build workspace
echo "Building entire workspace..."
if cargo build $RELEASE_FLAG; then
    echo "✅ Workspace build successful"
else
    echo "❌ Workspace build failed"
    exit 1
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Build Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Profile:  $PROFILE"
echo "Duration: ${DURATION}s"
echo

# Show library sizes
if [ "$PROFILE" == "release" ]; then
    LIB_DIR="target/release"
else
    LIB_DIR="target/debug"
fi

if [ -d "$LIB_DIR" ]; then
    echo "📦 Library Sizes:"
    find "$LIB_DIR" -maxdepth 1 -type f -name "*.so" -o -name "*.a" | while read -r lib; do
        echo "  $(basename "$lib"): $(du -h "$lib" | cut -f1)"
    done
fi

echo
echo "✅ Native build complete!"
