use anyhow::Result;
use claws_runtime_core::{
    runtime_name, ClawRuntime, ModelProvider, RuntimeAgent, RuntimeContext, RuntimeKind,
    RuntimePlan,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct OpenClawRuntime;

impl Default for OpenClawRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenClawRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl ClawRuntime for OpenClawRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::OpenClaw
    }

    fn name(&self) -> &'static str {
        runtime_name(self.kind())
    }

    fn supports_multi_agent(&self) -> bool {
        true
    }

    fn build_plan(&self, ctx: &RuntimeContext) -> Result<RuntimePlan> {
        let config_json = build_config_json(ctx)?;
        let onboard_command = generate_onboard_command(&ctx.primary)?;

        let mut env = BTreeMap::new();
        env.insert("OPENCLAW_AGENT_NAME".to_string(), ctx.primary.name.clone());
        env.insert(
            "OPENCLAW_CONFIG".to_string(),
            serde_json::to_string(&config_json)?,
        );
        if let Some(onboard) = onboard_command {
            env.insert("OPENCLAW_ONBOARD_CMD".to_string(), onboard.join(" "));
        }
        if let Some(api_key) = &ctx.primary.model_api_key {
            env.insert("OPENCLAW_API_KEY".to_string(), api_key.clone());
        }
        if let Some(token) = &ctx.primary.discord_bot_token {
            env.insert("DISCORD_BOT_TOKEN".to_string(), token.clone());
        }

        let init_script = r#"#!/bin/bash
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
"#
        .to_string();

        Ok(RuntimePlan {
            env,
            init_script,
            services: Vec::new(),
        })
    }
}

fn generate_onboard_command(agent: &RuntimeAgent) -> Result<Option<Vec<String>>> {
    let mut args = vec!["onboard".to_string(), "--non-interactive".to_string()];

    match agent.model_provider {
        ModelProvider::Anthropic => {
            args.push("--model".to_string());
            args.push("anthropic".to_string());
        }
        ModelProvider::OpenAI => {
            args.push("--model".to_string());
            args.push("openai".to_string());
        }
        ModelProvider::BYOM => {
            args.push("--model".to_string());
            args.push("custom".to_string());
        }
        ModelProvider::OpenClaw => {}
    }

    if let Some(workspace_dir) = &agent.workspace_dir {
        args.push("--agent-dir".to_string());
        args.push(workspace_dir.clone());
    }

    Ok(Some(args))
}

fn build_config_json(ctx: &RuntimeContext) -> Result<Value> {
    let agents = if ctx.agents.is_empty() {
        vec![ctx.primary.clone()]
    } else {
        ctx.agents.clone()
    };

    let list: Vec<Value> = agents
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "workspace": a.workspace_dir.clone().unwrap_or_else(|| {
                    format!("~/.openclaw/workspace-{}", a.id)
                })
            })
        })
        .collect();

    let mut config = json!({
        "agents": {
            "list": list
        }
    });

    let mut all_bindings = Vec::new();
    let mut bot_token: Option<String> = None;

    for a in &agents {
        if let Some(token) = &a.discord_bot_token {
            bot_token.get_or_insert_with(|| token.clone());
        }
        if let Some(channels) = &a.discord_channels {
            all_bindings.push(channel_binding(
                channels.coordination_logs.clone(),
                &a.id,
                "coordination_logs",
            ));
            all_bindings.push(channel_binding(
                channels.slave_communication.clone(),
                &a.id,
                "slave_communication",
            ));
            all_bindings.push(channel_binding(
                channels.master_orders.clone(),
                &a.id,
                "master_orders",
            ));
        } else if let Some(channel_id) = &a.discord_channel_id {
            all_bindings.push(json!({
                "channelId": channel_id,
                "agentId": a.id,
            }));
        }
    }

    if let Some(token) = bot_token {
        if !all_bindings.is_empty() {
            config["channels"] = json!({
                "discord": {
                    "token": token,
                    "bindings": all_bindings,
                }
            });
        }
    }

    let first = agents.first().unwrap_or(&ctx.primary);
    match first.model_provider {
        ModelProvider::Anthropic => {
            if let Some(api_key) = &first.model_api_key {
                config["auth"] = json!({
                    "anthropic": { "apiKey": api_key }
                });
            }
        }
        ModelProvider::OpenAI => {
            if let Some(api_key) = &first.model_api_key {
                config["auth"] = json!({
                    "openai": { "apiKey": api_key }
                });
            }
        }
        ModelProvider::BYOM => {
            if let Some(endpoint) = &first.model_endpoint {
                config["models"] = json!({
                    "custom": {
                        "endpoint": endpoint,
                        "apiKey": first.model_api_key,
                    }
                });
            }
        }
        ModelProvider::OpenClaw => {}
    }

    if let Some(runtime_config) = &first.runtime_config {
        merge_json(&mut config, runtime_config);
    }

    Ok(config)
}

fn channel_binding(channel_id: String, agent_id: &str, purpose: &str) -> Value {
    json!({
        "channelId": channel_id,
        "agentId": agent_id,
        "purpose": purpose,
    })
}

fn merge_json(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                merge_json(base_map.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (base_slot, overlay_val) => {
            *base_slot = overlay_val.clone();
        }
    }
}
