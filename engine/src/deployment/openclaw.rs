use crate::models::{Agent, ModelProvider};
use anyhow::Result;
use serde_json::json;
use std::process::Command;

pub struct OpenClawConfig {
    pub agent: Agent,
}

impl OpenClawConfig {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    pub fn generate_onboard_command(&self) -> Result<Vec<String>> {
        let mut args = vec!["onboard".to_string(), "--non-interactive".to_string()];

        // Model provider
        match &self.agent.model_provider {
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
            ModelProvider::OpenClaw => {
                // Use default OpenClaw model
            }
        }

        // Agent directory
        if let Some(workspace_dir) = &self.agent.workspace_dir {
            args.push("--agent-dir".to_string());
            args.push(workspace_dir.clone());
        }

        Ok(args)
    }

    pub fn generate_config_json(&self) -> Result<serde_json::Value> {
        let mut config = json!({
            "agents": {
                "list": [{
                    "name": self.agent.name,
                    "workspace": self.agent.workspace_dir.clone().unwrap_or_else(|| {
                        format!("~/.openclaw/workspace-{}", self.agent.id)
                    }),
                }]
            }
        });

        // Add Discord channel bindings if configured
        if let Some(bot_token) = &self.agent.discord_bot_token {
            let mut bindings = Vec::new();

            // Use agent-specific channels if provided, otherwise fall back to team channels
            if let Some(agent_channels) = &self.agent.discord_channels {
                // Bind to all three channel types for comprehensive communication
                bindings.push(json!({
                    "channelId": agent_channels.coordination_logs,
                    "agentId": self.agent.id.to_string(),
                    "purpose": "coordination_logs"
                }));
                bindings.push(json!({
                    "channelId": agent_channels.slave_communication,
                    "agentId": self.agent.id.to_string(),
                    "purpose": "slave_communication"
                }));
                bindings.push(json!({
                    "channelId": agent_channels.master_orders,
                    "agentId": self.agent.id.to_string(),
                    "purpose": "master_orders"
                }));
            } else if let Some(channel_id) = &self.agent.discord_channel_id {
                // Fallback to single channel (legacy support)
                bindings.push(json!({
                    "channelId": channel_id,
                    "agentId": self.agent.id.to_string(),
                }));
            }

            if !bindings.is_empty() {
                config["channels"] = json!({
                    "discord": {
                        "token": bot_token,
                        "bindings": bindings
                    }
                });
            }
        }

        // Add model configuration
        match &self.agent.model_provider {
            ModelProvider::Anthropic => {
                if let Some(api_key) = &self.agent.model_api_key {
                    config["auth"] = json!({
                        "anthropic": {
                            "apiKey": api_key
                        }
                    });
                }
            }
            ModelProvider::OpenAI => {
                if let Some(api_key) = &self.agent.model_api_key {
                    config["auth"] = json!({
                        "openai": {
                            "apiKey": api_key
                        }
                    });
                }
            }
            ModelProvider::BYOM => {
                if let Some(endpoint) = &self.agent.model_endpoint {
                    config["models"] = json!({
                        "custom": {
                            "endpoint": endpoint,
                            "apiKey": self.agent.model_api_key
                        }
                    });
                }
            }
            ModelProvider::OpenClaw => {
                // Use default OpenClaw configuration
            }
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
