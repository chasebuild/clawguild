use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::models::{Agent, DeploymentStatus};

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub agent: Agent,
    #[allow(dead_code)]
    pub region: Option<String>,
    pub openclaw_onboard_command: Option<Vec<String>>,
    pub openclaw_config_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct DeploymentId {
    #[allow(dead_code)]
    pub id: Uuid,
    pub provider_id: String,
}

#[derive(Debug, Clone)]
pub struct VpsAgentStatus {
    #[allow(dead_code)]
    pub deployment_id: DeploymentId,
    pub status: DeploymentStatus,
    pub endpoint: Option<String>,
    pub gateway_url: Option<String>,
}

#[async_trait]
pub trait VpsProvider: Send + Sync {
    async fn deploy_agent(&self, config: AgentConfig) -> Result<DeploymentId>;
    async fn get_status(&self, deployment_id: &DeploymentId) -> Result<VpsAgentStatus>;
    async fn destroy_agent(&self, deployment_id: &DeploymentId) -> Result<()>;
    async fn update_config(&self, deployment_id: &DeploymentId, config: AgentConfig) -> Result<()>;
    fn provider_name(&self) -> &str;
}
