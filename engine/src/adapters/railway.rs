use crate::adapters::trait_def::{AgentConfig, DeploymentId, VpsAgentStatus, VpsProvider};
use crate::config::Config;
use crate::models::DeploymentStatus;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

pub struct RailwayAdapter {
    client: Client,
    api_key: String,
}

impl RailwayAdapter {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .railway_api_key
            .as_ref()
            .context("Railway API key not configured")?
            .clone();

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }
}

#[async_trait]
impl VpsProvider for RailwayAdapter {
    async fn deploy_agent(&self, config: AgentConfig) -> Result<DeploymentId> {
        // Railway API: Create project
        let project_response = self
            .client
            .post("https://api.railway.app/v1/projects")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "name": format!("clawguild-{}", config.agent.name)
            }))
            .send()
            .await?;

        let project: serde_json::Value = project_response.json().await?;
        let project_id = project["project"]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get project ID"))?;

        // Prepare service configuration with OpenClaw setup
        let mut service_config = serde_json::json!({
            "name": config.agent.name,
            "source": {
                "repo": "openclaw/openclaw",
                "template": "openclaw"
            }
        });

        // Build environment variables object
        let mut env_vars = serde_json::Map::new();

        // Add OpenClaw configuration as environment variable
        if let Some(config_json) = &config.openclaw_config_json {
            let config_str = serde_json::to_string(config_json)?;
            env_vars.insert(
                "OPENCLAW_CONFIG".to_string(),
                serde_json::Value::String(config_str),
            );
        }

        // Add onboard command as environment variable
        if let Some(onboard_cmd) = &config.openclaw_onboard_command {
            let cmd_str = onboard_cmd.join(" ");
            env_vars.insert(
                "OPENCLAW_ONBOARD_CMD".to_string(),
                serde_json::Value::String(cmd_str),
            );
        }

        // Add environment variables to service config if any were set
        if !env_vars.is_empty() {
            service_config["variables"] = serde_json::Value::Object(env_vars);
        }

        // Create service with OpenClaw template
        let service_response = self
            .client
            .post(&format!(
                "https://api.railway.app/v1/projects/{}/services",
                project_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&service_config)
            .send()
            .await?;

        let service: serde_json::Value = service_response.json().await?;
        let service_id = service["service"]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get service ID"))?;

        let provider_id = format!("railway-{}", service_id);

        Ok(DeploymentId {
            id: config.agent.deployment_id.unwrap_or_else(Uuid::new_v4),
            provider_id,
        })
    }

    async fn get_status(&self, deployment_id: &DeploymentId) -> Result<VpsAgentStatus> {
        // Extract service ID from provider_id
        let service_id = deployment_id
            .provider_id
            .strip_prefix("railway-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;

        let response = self
            .client
            .get(&format!(
                "https://api.railway.app/v1/services/{}",
                service_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let service: serde_json::Value = response.json().await?;
        let status_str = service["service"]["status"].as_str().unwrap_or("unknown");

        let status = match status_str {
            "DEPLOYED" | "RUNNING" => DeploymentStatus::Running,
            "DEPLOYING" | "BUILDING" => DeploymentStatus::Creating,
            "FAILED" | "CRASHED" => DeploymentStatus::Failed,
            _ => DeploymentStatus::Pending,
        };

        let endpoint = service["service"]["url"].as_str().map(|s| s.to_string());

        Ok(VpsAgentStatus {
            deployment_id: deployment_id.clone(),
            status,
            endpoint: endpoint.clone(),
            gateway_url: endpoint.map(|url| format!("{}/openclaw", url)),
        })
    }

    async fn destroy_agent(&self, deployment_id: &DeploymentId) -> Result<()> {
        let service_id = deployment_id
            .provider_id
            .strip_prefix("railway-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;

        self.client
            .delete(&format!(
                "https://api.railway.app/v1/services/{}",
                service_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(())
    }

    async fn update_config(&self, deployment_id: &DeploymentId, config: AgentConfig) -> Result<()> {
        let service_id = deployment_id
            .provider_id
            .strip_prefix("railway-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;

        // Update environment variables
        let env_vars: serde_json::Value = serde_json::json!({
            "OPENCLAW_API_KEY": config.agent.model_api_key,
            "DISCORD_BOT_TOKEN": config.agent.discord_bot_token,
            "DISCORD_CHANNEL_ID": config.agent.discord_channel_id,
        });

        self.client
            .post(&format!(
                "https://api.railway.app/v1/services/{}/variables",
                service_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&env_vars)
            .send()
            .await?;

        Ok(())
    }

    async fn get_logs(&self, deployment_id: &DeploymentId, lines: Option<usize>) -> Result<Vec<String>> {
        let service_id = deployment_id
            .provider_id
            .strip_prefix("railway-")
            .ok_or_else(|| anyhow::anyhow!("Invalid provider ID"))?;

        let limit = lines.unwrap_or(100);

        // Railway API: Get deployment logs
        let response = self
            .client
            .get(&format!(
                "https://api.railway.app/v1/services/{}/logs?limit={}",
                service_id, limit
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec!["Logs not available via API. Use Railway dashboard.".to_string()]);
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
        "railway"
    }
}
