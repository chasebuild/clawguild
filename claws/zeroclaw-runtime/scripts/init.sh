#!/bin/bash
set -e

echo "Setting up ZeroClaw on VPS..."

apt-get update -y
apt-get install -y curl git build-essential pkg-config libssl-dev

if ! command -v cargo &> /dev/null; then
    echo "Installing Rust toolchain..."
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    source "$HOME/.cargo/env"
fi

if ! command -v zeroclaw &> /dev/null; then
    echo "Installing ZeroClaw..."
    source "$HOME/.cargo/env"
    cargo install --git https://github.com/theonlyhennygod/zeroclaw
fi

if [ -n "$ZEROCLAW_MODEL_PROVIDER" ]; then
    zeroclaw config set --model_provider "$ZEROCLAW_MODEL_PROVIDER"
fi
if [ -n "$ZEROCLAW_MODEL" ]; then
    zeroclaw config set --model "$ZEROCLAW_MODEL"
fi
if [ -n "$ZEROCLAW_API_KEY" ]; then
    zeroclaw config set --api_key "$ZEROCLAW_API_KEY"
fi
if [ -n "$ZEROCLAW_DISCORD_TOKEN" ]; then
    zeroclaw auth discord "$ZEROCLAW_DISCORD_TOKEN"
fi

echo "Creating ZeroClaw systemd service..."
cat > /etc/systemd/system/zeroclaw.service << 'SERVICEEOF'
[Unit]
Description=ZeroClaw Agent Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=/root/.cargo/bin/zeroclaw gateway
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICEEOF

systemctl daemon-reload
systemctl enable zeroclaw

echo "Starting ZeroClaw service..."
systemctl start zeroclaw

echo "ZeroClaw setup complete!"
systemctl status zeroclaw --no-pager
