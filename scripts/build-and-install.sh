#!/usr/bin/env bash
# Build hegel in release mode and install to ~/.cargo/bin
# Usage: ./scripts/build-and-install.sh [--skip-bump]

set -e

# Parse arguments
BUMP_VERSION=true
if [[ "$1" == "--skip-bump" ]]; then
    BUMP_VERSION=false
fi

# Read current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ "$BUMP_VERSION" = true ]; then
    echo "📌 Current version: $CURRENT_VERSION"

    # Parse version components
    IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

    # Increment patch version
    NEW_PATCH=$((PATCH + 1))
    NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"

    echo "⬆️  Incrementing to: $NEW_VERSION"

    # Update Cargo.toml with new version
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

    # Update Cargo.lock to reflect new version
    cargo update -p hegel --quiet

    BUILD_VERSION="$NEW_VERSION"
else
    BUILD_VERSION="$CURRENT_VERSION"
fi

echo "🔨 Building hegel v$BUILD_VERSION (release mode with bundled ast-grep)..."
cargo build --release --features bundle-ast-grep

echo ""
./scripts/post-build.sh

echo ""
echo "✨ Done! Run 'hegel --version' to verify."
if [ "$BUMP_VERSION" = true ]; then
    echo "📝 Version updated: $CURRENT_VERSION → $NEW_VERSION"
fi
