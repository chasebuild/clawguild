use anyhow::Result;
use claws_runtime_core::{runtime_name, ClawRuntime, RuntimeContext, RuntimeKind, RuntimePlan};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct NanoClawRuntime;

impl Default for NanoClawRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl NanoClawRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl ClawRuntime for NanoClawRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::NanoClaw
    }

    fn name(&self) -> &'static str {
        runtime_name(self.kind())
    }

    fn build_plan(&self, ctx: &RuntimeContext) -> Result<RuntimePlan> {
        let settings = runtime_settings(&ctx.primary);

        let mut env = BTreeMap::new();
        if let Some(api_key) = settings.anthropic_api_key {
            env.insert("NANOCLAW_ANTHROPIC_API_KEY".to_string(), api_key);
        }
        if let Some(args) = settings.run_args {
            env.insert("NANOCLAW_RUN_ARGS".to_string(), args);
        }
        if let Some(repo_ref) = settings.repo_ref {
            env.insert("NANOCLAW_REF".to_string(), repo_ref);
        }

        let init_script = r#"#!/bin/bash
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
"#
        .to_string();

        Ok(RuntimePlan {
            env,
            init_script,
            services: Vec::new(),
        })
    }
}

struct NanoClawSettings {
    anthropic_api_key: Option<String>,
    run_args: Option<String>,
    repo_ref: Option<String>,
}

fn runtime_settings(agent: &claws_runtime_core::RuntimeAgent) -> NanoClawSettings {
    let mut anthropic_api_key = agent.model_api_key.clone();
    let mut run_args = None;
    let mut repo_ref = None;

    if let Some(Value::Object(map)) = agent.runtime_config.as_ref() {
        if let Some(Value::String(value)) = map.get("anthropic_api_key") {
            anthropic_api_key = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("run_args") {
            run_args = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("repo_ref") {
            repo_ref = Some(value.clone());
        }
    }

    NanoClawSettings {
        anthropic_api_key,
        run_args,
        repo_ref,
    }
}
