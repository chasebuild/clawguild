#!/bin/bash
set -e

echo "Setting up PicoClaw on VPS..."

apt-get update -y
apt-get install -y curl git build-essential pkg-config libssl-dev

if ! command -v cargo &> /dev/null; then
    echo "Installing Rust toolchain..."
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    source "$HOME/.cargo/env"
fi

if ! command -v picoclaw &> /dev/null; then
    echo "Installing PicoClaw..."
    source "$HOME/.cargo/env"
    cargo install picoclaw
fi

if [ -n "$PICOCLAW_OPENROUTER_API_KEY" ]; then
    picoclaw config set openrouter_api_key "$PICOCLAW_OPENROUTER_API_KEY"
fi
if [ -n "$PICOCLAW_DISCORD_TOKEN" ]; then
    picoclaw discord login --token "$PICOCLAW_DISCORD_TOKEN"
fi

echo "Creating PicoClaw systemd service..."
cat > /etc/systemd/system/picoclaw.service << 'SERVICEEOF'
[Unit]
Description=PicoClaw Agent Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=/root/.cargo/bin/picoclaw daemon
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICEEOF

systemctl daemon-reload
systemctl enable picoclaw

echo "Starting PicoClaw service..."
systemctl start picoclaw

echo "PicoClaw setup complete!"
systemctl status picoclaw --no-pager
