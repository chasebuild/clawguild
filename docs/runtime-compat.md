# Claw Runtime Compatibility

ClawGuild now supports multiple runtime families behind the same orchestration surface. Each agent picks a `runtime` and optional `runtime_config` payload that controls runtime-specific setup on the VPS.

## Runtime Matrix

- `openclaw` (multi-agent supported)
- `zeroclaw`
- `picoclaw`
- `nanoclaw` (containerized runner, no Discord bridge yet)

## Agent Schema Additions

- `runtime`: `openclaw | zeroclaw | picoclaw | nanoclaw`
- `runtime_config`: JSON object for runtime-specific overrides

## Discord Routing

ClawGuild continues to route tasks via Discord. The runtime adapter will:

- Use `discord_bot_token` for bot login
- Bind channels via `discord_channels` (or legacy `discord_channel_id`) where supported

## Runtime Config Overrides

`runtime_config` is merged or interpreted per runtime. Examples below show only the keys that the runtime adapters recognize.

### OpenClaw

OpenClaw uses the generated `openclaw.json` based on agent fields. You can override or extend with `runtime_config`:

```json
{
  "channels": {
    "discord": {
      "token": "...",
      "bindings": []
    },
    "telegram": {
      "enabled": true,
      "botToken": "...",
      "dmPolicy": "pairing",
      "groups": { "*": { "requireMention": true } }
    }
  },
  "models": {
    "custom": { "endpoint": "...", "apiKey": "..." }
  }
}
```

### ZeroClaw

```json
{
  "model_provider": "openai",
  "model": "gpt-4o-mini",
  "api_key": "...",
  "discord_token": "..."
}
```

### PicoClaw

```json
{
  "openrouter_api_key": "...",
  "discord_token": "..."
}
```

### NanoClaw

```json
{
  "anthropic_api_key": "...",
  "run_args": "--some-flag",
  "repo_ref": "main"
}
```

## Multi-Agent Deployments

Only `openclaw` supports multi-agent deployments on a single VPS. Other runtimes must be deployed per agent.

## VPS Adapter Notes

- Fly.io: full runtime support via init scripts
- Railway: OpenClaw only (runtime validation enforced)
- AWS: placeholder adapter (runtime-aware logs only)
