#!/bin/bash
# Build script for Coriolis - creates fully portable binary with NO system dependencies!

set -e

echo "🦀 Building Coriolis portable binary (with static libraries)..."
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo ""
    echo "Install it with:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✓ Rust is installed ($(rustc --version))"

# Determine target based on OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    TARGET="x86_64-unknown-linux-musl"
    echo ""
    echo "Building static Linux binary (NO system dependencies!)..."
    echo ""

    # Check for musl-tools
    if ! command -v musl-gcc &> /dev/null; then
        echo "⚠️  musl-tools not found (needed for static builds)"
        echo ""
        echo "Install it with:"
        echo "  sudo apt-get install musl-tools"
        echo ""
        echo "Or build with system libraries instead:"
        echo "  cargo build --release"
        echo ""
        exit 1
    fi

    # Install target if needed
    if ! rustup target list | grep -q "$TARGET (installed)"; then
        echo "Installing $TARGET target..."
        rustup target add $TARGET
    fi

    # Build with static libraries
    echo "Building... (this may take 10-15 minutes the first time)"
    echo ""
    PKG_CONFIG_ALL_STATIC=1 cargo build --release --target $TARGET

    # Strip for smaller size
    echo ""
    echo "Stripping symbols..."
    strip target/$TARGET/release/coriolis 2>/dev/null || true

    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📦 Binary: ./target/$TARGET/release/coriolis"
    echo "📊 Size: $(du -h target/$TARGET/release/coriolis | cut -f1)"
    echo ""
    echo "Verifying it's static..."
    if ldd target/$TARGET/release/coriolis 2>&1 | grep -q "not a dynamic executable"; then
        echo "✅ Binary is fully static - NO dependencies!"
    else
        echo "⚠️  Binary has some dynamic dependencies:"
        ldd target/$TARGET/release/coriolis
    fi
    echo ""
    echo "🚀 This binary will run on ANY Linux system!"

elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo ""
    echo "Building macOS binary..."
    echo ""

    cargo build --release

    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📦 Binary: ./target/release/coriolis"
    echo "📊 Size: $(du -h target/release/coriolis | cut -f1)"

else
    # Windows or other
    echo ""
    echo "For Windows, use cross-compilation from Linux:"
    echo ""
    echo "  rustup target add x86_64-pc-windows-gnu"
    echo "  sudo apt-get install mingw-w64"
    echo "  PKG_CONFIG_ALL_STATIC=1 cargo build --release --target x86_64-pc-windows-gnu"
    echo ""
    exit 1
fi

echo ""
echo "To run:"
echo "  ./target/release/coriolis your_file.nc"
echo ""
echo "To install system-wide:"
echo "  sudo cp target/$TARGET/release/coriolis /usr/local/bin/"
echo ""
