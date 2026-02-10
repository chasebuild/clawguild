use crate::adapters::trait_def::{AgentConfig, DeploymentId, VpsAgentStatus, VpsProvider};
use crate::config::Config;
use crate::models::DeploymentStatus;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

pub struct FlyIoAdapter {
    client: Client,
    api_token: String,
}

impl FlyIoAdapter {
    pub fn new(config: &Config) -> Result<Self> {
        let api_token = config
            .fly_api_token
            .as_ref()
            .context("Fly.io API token not configured")?
            .clone();

        Ok(Self {
            client: Client::new(),
            api_token,
        })
    }
}

#[async_trait]
impl VpsProvider for FlyIoAdapter {
    async fn deploy_agent(&self, config: AgentConfig) -> Result<DeploymentId> {
        // Fly.io API: Create app (if it doesn't exist)
        let app_name = match &config.agents {
            Some(agents) if agents.len() > 1 => format!(
                "clawguild-multi-{}",
                config.agent.name.to_lowercase().replace(' ', "-")
            ),
            _ => format!(
                "clawguild-{}",
                config.agent.name.to_lowercase().replace(' ', "-")
            ),
        };
        
        // Try to get existing app or create new one
        let app_response = self
            .client
            .get(&format!("https://api.machines.dev/v1/apps/{}", app_name))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await;

        let app_id = if let Ok(response) = app_response {
            if response.status().is_success() {
                let app: serde_json::Value = response.json().await?;
                app["id"]
                    .as_str()
                    .or_else(|| app["name"].as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("Failed to get app ID"))
            } else {
                // App doesn't exist, create it
                let create_response = self
                    .client
                    .post("https://api.machines.dev/v1/apps")
                    .header("Authorization", format!("Bearer {}", self.api_token))
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({
                        "app_name": app_name,
                        "org_slug": "personal"
                    }))
                    .send()
                    .await?;

                let app: serde_json::Value = create_response.json().await?;
                app["id"]
                    .as_str()
                    .or_else(|| app["name"].as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("Failed to get app ID"))
            }
        } else {
            // Create new app
            let create_response = self
                .client
                .post("https://api.machines.dev/v1/apps")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "app_name": app_name,
                "org_slug": "personal"
            }))
            .send()
            .await?;
        
            let app: serde_json::Value = create_response.json().await?;
            app["id"]
                .as_str()
            .or_else(|| app["name"].as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("Failed to get app ID"))
        }?;
        
        // Create a Fly.io machine (VM) - OpenClaw runs directly on the VM, not in Docker
        let region = config.region.as_deref().unwrap_or("iad"); // Default to iad (Washington, D.C.)

        // Build environment variables for OpenClaw configuration
        let mut env_vars = serde_json::Map::new();
        env_vars.insert(
            "OPENCLAW_AGENT_NAME".to_string(),
            serde_json::Value::String(config.agent.name.clone()),
        );

        // Store OpenClaw config as environment variable (will be written to file by init script)
        if let Some(config_json) = &config.openclaw_config_json {
            let config_str = serde_json::to_string(config_json)?;
            env_vars.insert(
                "OPENCLAW_CONFIG".to_string(),
                serde_json::Value::String(config_str),
            );
        }
        
        // Store onboarding command as environment variable
        if let Some(onboard_cmd) = &config.openclaw_onboard_command {
            let cmd_str = onboard_cmd.join(" ");
            env_vars.insert(
                "OPENCLAW_ONBOARD_CMD".to_string(),
                serde_json::Value::String(cmd_str),
            );
        }

        // Store Discord bot token if available
        if let Some(bot_token) = &config.agent.discord_bot_token {
            env_vars.insert(
                "DISCORD_BOT_TOKEN".to_string(),
                serde_json::Value::String(bot_token.clone()),
            );
        }

        // Store model API key if available
        if let Some(api_key) = &config.agent.model_api_key {
            env_vars.insert(
                "OPENCLAW_API_KEY".to_string(),
                serde_json::Value::String(api_key.clone()),
            );
        }
        
        // Create machine with init script that installs OpenClaw directly on the VM
        // OpenClaw runs directly on the VPS, not in a Docker container
        let init_script = r#"#!/bin/bash
set -e

echo "Setting up OpenClaw on VPS..."

# Install Node.js and npm if not present
if ! command -v node &> /dev/null; then
    echo "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
    apt-get install -y nodejs
fi

# Install OpenClaw CLI if not present
if ! command -v openclaw &> /dev/null; then
    echo "Installing OpenClaw CLI..."
    npm install -g openclaw
fi

# Create OpenClaw config directory
mkdir -p ~/.openclaw

# Write OpenClaw configuration if provided via environment variable
if [ -n "$OPENCLAW_CONFIG" ]; then
    echo "Writing OpenClaw configuration..."
    echo "$OPENCLAW_CONFIG" > ~/.openclaw/openclaw.json
fi

# Run onboarding if not already configured and command is provided
if [ ! -f ~/.openclaw/openclaw.json ] && [ -n "$OPENCLAW_ONBOARD_CMD" ]; then
    echo "Running OpenClaw onboarding..."
    eval "openclaw $OPENCLAW_ONBOARD_CMD"
elif [ -n "$OPENCLAW_ONBOARD_CMD" ]; then
    echo "OpenClaw already configured, skipping onboarding..."
fi

# Create systemd service for OpenClaw
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

# Reload systemd and enable service
systemctl daemon-reload
systemctl enable openclaw

# Start the service
echo "Starting OpenClaw service..."
systemctl start openclaw

echo "OpenClaw setup complete!"
systemctl status openclaw --no-pager
"#;

        let machine_config = serde_json::json!({
            "name": format!("{}-machine", app_name),
            "region": region,
            "config": {
                "image": "debian:bookworm-slim",
                "init": {
                    "cmd": ["/bin/bash", "-c", init_script]
                },
                "env": env_vars,
                "services": [
                    {
                        "ports": [
                            {
                                "port": 3000,
                                "handlers": ["http"],
                                "force_https": true
                            }
                        ],
                        "protocol": "tcp",
                        "internal_port": 3000
                    }
                ]
            }
        });

        let machine_response = self
            .client
            .post(&format!(
                "https://api.machines.dev/v1/apps/{}/machines",
                &app_id
            ))
                .header("Authorization", format!("Bearer {}", self.api_token))
                .header("Content-Type", "application/json")
            .json(&machine_config)
                .send()
                .await?;

        if !machine_response.status().is_success() {
            let error_text = machine_response.text().await?;
            anyhow::bail!("Failed to create Fly.io machine: {}", error_text);
        }

        let machine: serde_json::Value = machine_response.json().await?;
        let machine_id = machine["id"]
            .as_str()
            .or_else(|| machine["name"].as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to get machine ID"))?;
        
        let provider_id = format!("flyio-{}", machine_id);
        
        Ok(DeploymentId {
            id: config.agent.deployment_id.unwrap_or_else(Uuid::new_v4),
            provider_id,
        })
    }

    async fn get_status(&self, deployment_id: &DeploymentId) -> Result<VpsAgentStatus> {
        let app_id = deployment_id
            .provider_id
            .strip_prefix("flyio-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;
        
        let response = self
            .client
            .get(&format!("https://api.machines.dev/v1/apps/{}", app_id))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;
        
        let app: serde_json::Value = response.json().await?;
        let status_str = app["status"].as_str().unwrap_or("unknown");
        
        let status = match status_str {
            "running" | "started" => DeploymentStatus::Running,
            "starting" | "stopping" => DeploymentStatus::Creating,
            "stopped" | "failed" => DeploymentStatus::Failed,
            _ => DeploymentStatus::Pending,
        };
        
        let endpoint = app["hostname"].as_str().map(|s| format!("https://{}", s));
        
        Ok(VpsAgentStatus {
            deployment_id: deployment_id.clone(),
            status,
            endpoint: endpoint.clone(),
            gateway_url: endpoint.map(|url| format!("{}/openclaw", url)),
        })
    }

    async fn destroy_agent(&self, deployment_id: &DeploymentId) -> Result<()> {
        let app_id = deployment_id
            .provider_id
            .strip_prefix("flyio-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;
        
        self.client
            .delete(&format!("https://api.machines.dev/v1/apps/{}", app_id))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;
        
        Ok(())
    }

    async fn update_config(&self, deployment_id: &DeploymentId, config: AgentConfig) -> Result<()> {
        let app_id = deployment_id
            .provider_id
            .strip_prefix("flyio-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;
        
        // Update secrets (environment variables)
        let secrets: serde_json::Value = serde_json::json!({
            "OPENCLAW_API_KEY": config.agent.model_api_key,
            "DISCORD_BOT_TOKEN": config.agent.discord_bot_token,
            "DISCORD_CHANNEL_ID": config.agent.discord_channel_id,
        });
        
        self.client
            .post(&format!(
                "https://api.machines.dev/v1/apps/{}/secrets",
                app_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&secrets)
            .send()
            .await?;
        
        Ok(())
    }

    async fn get_logs(&self, deployment_id: &DeploymentId, lines: Option<usize>) -> Result<Vec<String>> {
        let machine_id = deployment_id
            .provider_id
            .strip_prefix("flyio-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;

        // Extract app name from machine ID (format: app-name-machine-id)
        // For now, we'll use the machine ID directly
        let limit = lines.unwrap_or(100);
        
        // Fly.io API: Get machine logs
        let response = self
            .client
            .get(&format!(
                "https://api.machines.dev/v1/apps/{}/machines/{}/logs?limit={}",
                "clawguild", // This should be extracted from deployment_id or stored
                machine_id,
                limit
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if !response.status().is_success() {
            // If logs endpoint doesn't exist, return empty or use alternative method
            return Ok(vec!["Logs not available via API. Use 'fly logs' CLI command.".to_string()]);
        }

        let logs: serde_json::Value = response.json().await?;
        let log_lines = logs["logs"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);

        Ok(log_lines)
    }

    fn provider_name(&self) -> &str {
        "flyio"
    }
}
