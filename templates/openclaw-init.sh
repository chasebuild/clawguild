#!/bin/bash
set -e

# This script initializes OpenClaw on a VPS instance
# It should be called with environment variables set for non-interactive setup

echo "Initializing OpenClaw..."

# Check if OpenClaw is installed
if ! command -v openclaw &> /dev/null; then
    echo "Installing OpenClaw..."
    npm install -g openclaw
fi

# Run onboarding if not already configured
if [ ! -f ~/.openclaw/openclaw.json ]; then
    echo "Running OpenClaw onboarding..."
    
    # Build command arguments
    ARGS=("onboard" "--non-interactive")
    
    if [ -n "$OPENCLAW_MODEL" ]; then
        ARGS+=("--model" "$OPENCLAW_MODEL")
    fi
    
    if [ -n "$OPENCLAW_AGENT_DIR" ]; then
        ARGS+=("--agent-dir" "$OPENCLAW_AGENT_DIR")
    fi
    
    openclaw "${ARGS[@]}"
fi

# Start OpenClaw gateway
echo "Starting OpenClaw gateway..."
openclaw start
