#!/bin/bash
set -euo pipefail

STUB_DIR="${STUB_DIR:-/tmp/runtime-stubs/bin}"
mkdir -p "$STUB_DIR"

write_stub() {
  local name="$1"
  local body="$2"
  printf '%s\n' "#!/bin/bash" "set -e" "$body" > "$STUB_DIR/$name"
  chmod +x "$STUB_DIR/$name"
}

write_stub "apt-get" 'exit 0'
write_stub "curl" 'exit 0'
write_stub "node" 'exit 0'
write_stub "npm" 'exit 0'
write_stub "openclaw" 'echo "$@" >> /tmp/openclaw.log; exit 0'
write_stub "cargo" 'exit 0'
write_stub "rustup" 'exit 0'
write_stub "zeroclaw" 'exit 0'
write_stub "picoclaw" 'exit 0'

write_stub "systemctl" 'echo "$@" >> /tmp/systemctl.log; exit 0'

write_stub "git" '
if [ "$1" = "clone" ]; then
  target="${@: -1}"
  mkdir -p "$target"
  exit 0
fi
exit 0
'

write_stub "docker" '
echo "$@" >> /tmp/docker.log
exit 0
'

if [ ! -x /usr/bin/docker ]; then
  install -m 0755 "$STUB_DIR/docker" /usr/bin/docker
fi
