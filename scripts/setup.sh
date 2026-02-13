#!/bin/bash
set -e

echo "Setting up ClawGuild..."

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Please install Rust first: https://rustup.rs/"
    exit 1
fi

# Build Rust API server (and engine)
echo "Building API server..."
cd api-server
cargo build --release
cd ..

# Setup dashboard
echo "Setting up dashboard..."
cd dashboard
if [ ! -d "node_modules" ]; then
    pnpm install
fi
cd ..

echo "Setup complete!"
echo ""
echo "To start the API server:"
echo "  cd api-server && cargo run"
echo ""
echo "To start the dashboard:"
echo "  cd dashboard && pnpm dev"
