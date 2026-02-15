#!/bin/bash
set -e

echo "Setting up OpenClaw on VPS..."

if ! command -v node &> /dev/null; then
    echo "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
    apt-get install -y nodejs
fi

if ! command -v openclaw &> /dev/null; then
    echo "Installing OpenClaw CLI..."
    npm install -g openclaw
fi

mkdir -p ~/.openclaw

if [ -n "$OPENCLAW_CONFIG" ]; then
    echo "Writing OpenClaw configuration..."
    echo "$OPENCLAW_CONFIG" > ~/.openclaw/openclaw.json
fi

if [ ! -f ~/.openclaw/openclaw.json ] && [ -n "$OPENCLAW_ONBOARD_CMD" ]; then
    echo "Running OpenClaw onboarding..."
    eval "openclaw $OPENCLAW_ONBOARD_CMD"
elif [ -n "$OPENCLAW_ONBOARD_CMD" ]; then
    echo "OpenClaw already configured, skipping onboarding..."
fi

echo "Creating OpenClaw systemd service..."
cat > /etc/systemd/system/openclaw.service << 'SERVICEEOF'
[Unit]
Description=OpenClaw Agent Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root
ExecStart=/usr/bin/openclaw start
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICEEOF

systemctl daemon-reload
systemctl enable openclaw

echo "Starting OpenClaw service..."
systemctl start openclaw

echo "OpenClaw setup complete!"
systemctl status openclaw --no-pager
