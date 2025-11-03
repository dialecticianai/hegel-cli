#!/usr/bin/env bash
# Build hegel in release mode (optionally install to ~/.cargo/bin)
# Usage:
#   ./scripts/build.sh                    # Just build
#   ./scripts/build.sh --install          # Build, bump version, and install
#   ./scripts/build.sh --install --skip-bump  # Build and install without version bump

set -e

# Parse arguments
INSTALL=false
BUMP_VERSION=false

for arg in "$@"; do
    case $arg in
        --install)
            INSTALL=true
            BUMP_VERSION=true  # Default to bumping when installing
            ;;
        --skip-bump)
            BUMP_VERSION=false
            ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 [--install] [--skip-bump]"
            exit 1
            ;;
    esac
done

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

if [ "$INSTALL" = true ]; then
    echo ""
    ./scripts/post-build.sh
fi

echo ""
echo "✨ Done! Binary: ./target/release/hegel"
if [ "$BUMP_VERSION" = true ]; then
    echo "📝 Version updated: $CURRENT_VERSION → $NEW_VERSION"
fi
if [ "$INSTALL" = true ]; then
    echo "✅ Installed to ~/.cargo/bin/hegel"
fi
