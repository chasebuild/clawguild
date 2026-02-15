use anyhow::Result;
use claws_runtime_core::{
    runtime_name, ClawRuntime, ModelProvider, RuntimeContext, RuntimeKind, RuntimePlan,
};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct ZeroClawRuntime;

impl Default for ZeroClawRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroClawRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl ClawRuntime for ZeroClawRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::ZeroClaw
    }

    fn name(&self) -> &'static str {
        runtime_name(self.kind())
    }

    fn build_plan(&self, ctx: &RuntimeContext) -> Result<RuntimePlan> {
        let settings = runtime_settings(&ctx.primary);

        let mut env = BTreeMap::new();
        if let Some(provider) = settings.model_provider {
            env.insert("ZEROCLAW_MODEL_PROVIDER".to_string(), provider);
        }
        if let Some(model) = settings.model {
            env.insert("ZEROCLAW_MODEL".to_string(), model);
        }
        if let Some(api_key) = settings.api_key {
            env.insert("ZEROCLAW_API_KEY".to_string(), api_key);
        }
        if let Some(token) = settings.discord_token {
            env.insert("ZEROCLAW_DISCORD_TOKEN".to_string(), token);
        }

        let init_script = r#"#!/bin/bash
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
"#
        .to_string();

        Ok(RuntimePlan {
            env,
            init_script,
            services: Vec::new(),
        })
    }
}

struct ZeroClawSettings {
    model_provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    discord_token: Option<String>,
}

fn runtime_settings(agent: &claws_runtime_core::RuntimeAgent) -> ZeroClawSettings {
    let mut model_provider = match agent.model_provider {
        ModelProvider::OpenAI => Some("openai".to_string()),
        ModelProvider::Anthropic => Some("anthropic".to_string()),
        ModelProvider::BYOM => Some("openrouter".to_string()),
        ModelProvider::OpenClaw => None,
    };
    let mut model = match agent.model_provider {
        ModelProvider::OpenAI => Some("gpt-4o-mini".to_string()),
        ModelProvider::Anthropic => Some("claude-3-5-sonnet-20240620".to_string()),
        ModelProvider::BYOM => Some("openrouter/auto".to_string()),
        ModelProvider::OpenClaw => None,
    };
    let mut api_key = agent.model_api_key.clone();
    let mut discord_token = agent.discord_bot_token.clone();

    if let Some(Value::Object(map)) = agent.runtime_config.as_ref() {
        if let Some(Value::String(value)) = map.get("model_provider") {
            model_provider = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("model") {
            model = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("api_key") {
            api_key = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("discord_token") {
            discord_token = Some(value.clone());
        }
    }

    ZeroClawSettings {
        model_provider,
        model,
        api_key,
        discord_token,
    }
}
