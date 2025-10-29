#!/usr/bin/env bash
# Build hegel in release mode and install to ~/.cargo/bin

set -e

echo "🔨 Building hegel (release mode with bundled ast-grep)..."
cargo build --release --features bundle-ast-grep

echo ""
./scripts/post-build.sh

echo ""
echo "✨ Done! Run 'hegel --version' to verify."
