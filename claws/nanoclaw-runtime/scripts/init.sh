#!/bin/bash
set -e

echo "Setting up NanoClaw on VPS..."

apt-get update -y
apt-get install -y git docker.io

systemctl enable docker
systemctl start docker

NANOCLAW_REF_VALUE=${NANOCLAW_REF:-main}

if [ ! -d /opt/nanoclaw ]; then
    git clone https://github.com/qwibitai/nanoclaw /opt/nanoclaw
fi

cd /opt/nanoclaw

git fetch origin

git checkout "$NANOCLAW_REF_VALUE"

git pull origin "$NANOCLAW_REF_VALUE"

echo "Building NanoClaw container..."
/usr/bin/docker build -t nanoclaw:latest -f container/Dockerfile .

cat > /etc/systemd/system/nanoclaw.service << 'SERVICEEOF'
[Unit]
Description=NanoClaw Agent Service
After=network.target docker.service
Requires=docker.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/nanoclaw
Environment=ANTHROPIC_API_KEY=${NANOCLAW_ANTHROPIC_API_KEY}
ExecStart=/usr/bin/docker run --rm \
  --name nanoclaw-agent \
  -e ANTHROPIC_API_KEY=${NANOCLAW_ANTHROPIC_API_KEY} \
  nanoclaw:latest ${NANOCLAW_RUN_ARGS}
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICEEOF

systemctl daemon-reload
systemctl enable nanoclaw

echo "Starting NanoClaw service..."
systemctl start nanoclaw

echo "NanoClaw setup complete!"
systemctl status nanoclaw --no-pager
