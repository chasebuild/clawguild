use anyhow::{Context, Result};
use async_trait::async_trait;
use crate::adapters::trait_def::{VpsProvider, AgentConfig, DeploymentId, VpsAgentStatus};
use crate::config::Config;
use crate::models::DeploymentStatus;
use reqwest::Client;
use uuid::Uuid;

pub struct FlyIoAdapter {
    client: Client,
    api_token: String,
}

impl FlyIoAdapter {
    pub fn new(config: &Config) -> Result<Self> {
        let api_token = config.fly_api_token
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
        // Fly.io API: Create app
        let app_name = format!("clawguild-{}", config.agent.name.to_lowercase().replace(' ', "-"));
        
        let response = self.client
            .post("https://api.machines.dev/v1/apps")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "app_name": app_name,
                "org_slug": "personal"
            }))
            .send()
            .await?;
        
        let app: serde_json::Value = response.json().await?;
        let app_id = app["id"].as_str()
            .or_else(|| app["name"].as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to get app ID"))?;
        
        // Set OpenClaw configuration and onboard command as secrets
        let mut secrets = serde_json::Map::new();
        
        if let Some(config_json) = &config.openclaw_config_json {
            let config_str = serde_json::to_string(config_json)?;
            secrets.insert("OPENCLAW_CONFIG".to_string(), serde_json::Value::String(config_str));
        }
        
        if let Some(onboard_cmd) = &config.openclaw_onboard_command {
            let cmd_str = onboard_cmd.join(" ");
            secrets.insert("OPENCLAW_ONBOARD_CMD".to_string(), serde_json::Value::String(cmd_str));
        }
        
        // Set secrets if any were configured
        if !secrets.is_empty() {
            self.client
                .post(&format!("https://api.machines.dev/v1/apps/{}/secrets", app_id))
                .header("Authorization", format!("Bearer {}", self.api_token))
                .header("Content-Type", "application/json")
                .json(&serde_json::Value::Object(secrets))
                .send()
                .await?;
        }
        
        let provider_id = format!("flyio-{}", app_id);
        
        Ok(DeploymentId {
            id: config.agent.deployment_id.unwrap_or_else(Uuid::new_v4),
            provider_id,
        })
    }

    async fn get_status(&self, deployment_id: &DeploymentId) -> Result<VpsAgentStatus> {
        let app_id = deployment_id.provider_id.strip_prefix("flyio-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;
        
        let response = self.client
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
        let app_id = deployment_id.provider_id.strip_prefix("flyio-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;
        
        self.client
            .delete(&format!("https://api.machines.dev/v1/apps/{}", app_id))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;
        
        Ok(())
    }

    async fn update_config(&self, deployment_id: &DeploymentId, config: AgentConfig) -> Result<()> {
        let app_id = deployment_id.provider_id.strip_prefix("flyio-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;
        
        // Update secrets (environment variables)
        let secrets: serde_json::Value = serde_json::json!({
            "OPENCLAW_API_KEY": config.agent.model_api_key,
            "DISCORD_BOT_TOKEN": config.agent.discord_bot_token,
            "DISCORD_CHANNEL_ID": config.agent.discord_channel_id,
        });
        
        self.client
            .post(&format!("https://api.machines.dev/v1/apps/{}/secrets", app_id))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&secrets)
            .send()
            .await?;
        
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "flyio"
    }
}
