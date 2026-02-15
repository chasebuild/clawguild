#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STUB_DIR="/tmp/runtime-stubs/bin"
export STUB_DIR

bash "$ROOT_DIR/scripts/ci/runtime-init-stubs/setup-stubs.sh"

export PATH="$STUB_DIR:$PATH"
export HOME="/root"

export DEBIAN_FRONTEND=noninteractive

mkdir -p /etc/systemd/system

run_script() {
  local name="$1"
  local script="$2"
  echo "==> Running $name init script"
  bash "$script"
}

assert_file() {
  local file="$1"
  if [ ! -f "$file" ]; then
    echo "Expected file not found: $file"
    exit 1
  fi
}

assert_systemctl_start() {
  local service="$1"
  if ! grep -q "start $service" /tmp/systemctl.log; then
    echo "Expected systemctl start for $service"
    exit 1
  fi
}

touch /tmp/systemctl.log

export OPENCLAW_ONBOARD_CMD="onboard --non-interactive"
unset OPENCLAW_CONFIG
run_script "openclaw" "$ROOT_DIR/claws/openclaw-runtime/scripts/init.sh"
assert_file "/etc/systemd/system/openclaw.service"
assert_systemctl_start "openclaw"
if ! grep -q "onboard" /tmp/openclaw.log; then
  echo "Expected openclaw onboarding to run."
  exit 1
fi

export ZEROCLAW_MODEL_PROVIDER="openai"
export ZEROCLAW_MODEL="gpt-4o-mini"
export ZEROCLAW_API_KEY="test"
run_script "zeroclaw" "$ROOT_DIR/claws/zeroclaw-runtime/scripts/init.sh"
assert_file "/etc/systemd/system/zeroclaw.service"
assert_systemctl_start "zeroclaw"

export PICOCLAW_OPENROUTER_API_KEY="test"
run_script "picoclaw" "$ROOT_DIR/claws/picoclaw-runtime/scripts/init.sh"
assert_file "/etc/systemd/system/picoclaw.service"
assert_systemctl_start "picoclaw"

export NANOCLAW_REF="main"
export NANOCLAW_ANTHROPIC_API_KEY="test"
export NANOCLAW_RUN_ARGS=""
run_script "nanoclaw" "$ROOT_DIR/claws/nanoclaw-runtime/scripts/init.sh"
assert_file "/etc/systemd/system/nanoclaw.service"
assert_systemctl_start "nanoclaw"

echo "All runtime init scripts completed successfully."
