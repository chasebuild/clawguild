#!/bin/bash
set -e

# This script sets up OpenClaw directly on a VPS instance (not in Docker)
# It installs OpenClaw CLI and configures it to run as a system service

echo "Setting up OpenClaw on VPS..."

# Install Node.js and npm if not present
if ! command -v node &> /dev/null; then
    echo "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
    apt-get install -y nodejs
fi

# Install OpenClaw CLI if not present
if ! command -v openclaw &> /dev/null; then
    echo "Installing OpenClaw CLI..."
    npm install -g openclaw
fi

# Create OpenClaw config directory
mkdir -p ~/.openclaw

# Write OpenClaw configuration if provided via environment variable
if [ -n "$OPENCLAW_CONFIG" ]; then
    echo "Writing OpenClaw configuration..."
    echo "$OPENCLAW_CONFIG" > ~/.openclaw/openclaw.json
fi

# Run onboarding if not already configured and command is provided
if [ ! -f ~/.openclaw/openclaw.json ] && [ -n "$OPENCLAW_ONBOARD_CMD" ]; then
    echo "Running OpenClaw onboarding..."
    # Split the command string and execute
    eval "openclaw $OPENCLAW_ONBOARD_CMD"
elif [ -n "$OPENCLAW_ONBOARD_CMD" ]; then
    echo "OpenClaw already configured, skipping onboarding..."
fi

# Create systemd service for OpenClaw
echo "Creating OpenClaw systemd service..."
cat > /etc/systemd/system/openclaw.service << 'EOF'
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
EOF

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable openclaw

# Start the service
echo "Starting OpenClaw service..."
systemctl start openclaw

echo "OpenClaw setup complete!"
systemctl status openclaw --no-pager
