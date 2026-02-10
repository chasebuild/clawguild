use crate::models::{Agent, ModelProvider};
use anyhow::Result;
use serde_json::json;
use std::process::Command;

pub struct OpenClawConfig {
    /// Single-agent mode: one agent.
    pub agent: Agent,
    /// Multi-agent mode: when set, config is generated for all agents on one VPS (same coordination).
    pub agents: Option<Vec<Agent>>,
}

impl OpenClawConfig {
    pub fn new(agent: Agent) -> Self {
        Self {
            agent: agent.clone(),
            agents: None,
        }
    }

    /// Build config for multiple agents on one VPS; coordination (Discord bindings) is per agent.
    pub fn multi(agents: Vec<Agent>) -> Self {
        let agent = agents.first().cloned().expect("multi requires at least one agent");
        Self {
            agent,
            agents: Some(agents),
        }
    }

    fn agents_slice(&self) -> &[Agent] {
        match &self.agents {
            Some(a) => a.as_slice(),
            None => std::slice::from_ref(&self.agent),
        }
    }

    pub fn generate_onboard_command(&self) -> Result<Vec<String>> {
        let first = self.agents_slice().first().unwrap_or(&self.agent);
        let mut args = vec!["onboard".to_string(), "--non-interactive".to_string()];

        match &first.model_provider {
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

        if let Some(workspace_dir) = &first.workspace_dir {
            args.push("--agent-dir".to_string());
            args.push(workspace_dir.clone());
        }

        Ok(args)
    }

    pub fn generate_config_json(&self) -> Result<serde_json::Value> {
        let agents_slice = self.agents_slice();
        let list: Vec<serde_json::Value> = agents_slice
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "workspace": a.workspace_dir.clone().unwrap_or_else(|| {
                        format!("~/.openclaw/workspace-{}", a.id)
                    }),
                })
            })
            .collect();

        let mut config = json!({
            "agents": {
                "list": list
            }
        });

        // Merge Discord bindings from all agents (same coordination mechanism per agent)
        let mut all_bindings = Vec::new();
        let mut bot_token: Option<String> = None;
        for a in agents_slice {
            if let Some(token) = &a.discord_bot_token {
                bot_token.get_or_insert_with(|| token.clone());
            }
            if let Some(channels) = &a.discord_channels {
                all_bindings.push(json!({
                    "channelId": channels.coordination_logs,
                    "agentId": a.id.to_string(),
                    "purpose": "coordination_logs"
                }));
                all_bindings.push(json!({
                    "channelId": channels.slave_communication,
                    "agentId": a.id.to_string(),
                    "purpose": "slave_communication"
                }));
                all_bindings.push(json!({
                    "channelId": channels.master_orders,
                    "agentId": a.id.to_string(),
                    "purpose": "master_orders"
                }));
            } else if let Some(channel_id) = &a.discord_channel_id {
                all_bindings.push(json!({
                    "channelId": channel_id,
                    "agentId": a.id.to_string(),
                }));
            }
        }
        if let Some(token) = bot_token {
            if !all_bindings.is_empty() {
                config["channels"] = json!({
                    "discord": {
                        "token": token,
                        "bindings": all_bindings
                    }
                });
            }
        }

        // Auth: use first agent's model config (multi-agent typically shares same provider)
        let first = agents_slice.first().unwrap_or(&self.agent);
        match &first.model_provider {
            ModelProvider::Anthropic => {
                if let Some(api_key) = &first.model_api_key {
                    config["auth"] = json!({
                        "anthropic": {
                            "apiKey": api_key
                        }
                    });
                }
            }
            ModelProvider::OpenAI => {
                if let Some(api_key) = &first.model_api_key {
                    config["auth"] = json!({
                        "openai": {
                            "apiKey": api_key
                        }
                    });
                }
            }
            ModelProvider::BYOM => {
                if let Some(endpoint) = &first.model_endpoint {
                    config["models"] = json!({
                        "custom": {
                            "endpoint": endpoint,
                            "apiKey": first.model_api_key
                        }
                    });
                }
            }
            ModelProvider::OpenClaw => {}
        }

        Ok(config)
    }

    pub async fn execute_onboard(&self) -> Result<()> {
        let args = self.generate_onboard_command()?;

        let output = Command::new("openclaw").args(&args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("OpenClaw onboard failed: {}", stderr);
        }

        Ok(())
    }
}
