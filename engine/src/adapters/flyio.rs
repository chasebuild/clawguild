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

        let mut env_vars = serde_json::Map::new();
        for (key, value) in &config.runtime_env {
            env_vars.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        let init_script = config.runtime_init_script.as_str();

        let mut machine_config = serde_json::json!({
            "name": format!("{}-machine", app_name),
            "region": region,
            "config": {
                "image": "debian:bookworm-slim",
                "init": {
                    "cmd": ["/bin/bash", "-c", init_script]
                },
                "env": env_vars
            }
        });

        if !config.runtime_services.is_empty() {
            let services: Vec<serde_json::Value> = config
                .runtime_services
                .iter()
                .map(|service| {
                    serde_json::json!({
                        "ports": [
                            {
                                "port": service.port,
                                "handlers": service.handlers.clone(),
                                "force_https": true
                            }
                        ],
                        "protocol": "tcp",
                        "internal_port": service.internal_port
                    })
                })
                .collect();

            machine_config["config"]["services"] = serde_json::Value::Array(services);
        }

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
            gateway_url: endpoint.clone(),
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
        
        let mut secrets_map = serde_json::Map::new();
        for (key, value) in &config.runtime_env {
            secrets_map.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        let secrets = serde_json::Value::Object(secrets_map);
        
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
