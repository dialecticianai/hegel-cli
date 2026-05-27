#!/usr/bin/env bash
# Post-build hook: Install release binary to ~/.cargo/bin after successful release build

set -e

RELEASE_BIN="./target/release/hegel"
INSTALL_DIR="$HOME/.cargo/bin"

# Only run if release binary exists and is newer than installed version
if [ -f "$RELEASE_BIN" ]; then
    # Check if we should install (binary is newer or doesn't exist in install dir)
    if [ ! -f "$INSTALL_DIR/hegel" ] || [ "$RELEASE_BIN" -nt "$INSTALL_DIR/hegel" ]; then
        echo "📦 Installing hegel to $INSTALL_DIR..."
        mkdir -p "$INSTALL_DIR"
        # Remove the old binary first so the copy lands on a fresh inode. On macOS,
        # cp-ing over a running/cached signed binary can invalidate the kernel's
        # code-signature cache for that path and SIGKILL it on launch ("Killed: 9").
        rm -f "$INSTALL_DIR/hegel"
        cp "$RELEASE_BIN" "$INSTALL_DIR/hegel"
        chmod +x "$INSTALL_DIR/hegel"
        echo "✅ Installed: $(hegel --version 2>/dev/null || echo 'hegel (version unknown)')"
    fi
fi
