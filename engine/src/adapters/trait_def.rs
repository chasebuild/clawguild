use crate::models::{Agent, DeploymentStatus};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Primary agent (used for single-agent deploy and for app naming when multi).
    pub agent: Agent,
    /// When set with len > 1, this deployment hosts multiple agents on one VPS.
    pub agents: Option<Vec<Agent>>,
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
    async fn get_logs(&self, deployment_id: &DeploymentId, lines: Option<usize>) -> Result<Vec<String>>;
    fn provider_name(&self) -> &str;
}
