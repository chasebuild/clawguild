use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub deployment_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub discord_bot_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub model_provider: ModelProvider,
    pub model_api_key: Option<String>,
    pub model_endpoint: Option<String>,
    pub personality: Option<String>,
    pub skills: Vec<String>,
    pub workspace_dir: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    Master,
    Slave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Pending,
    Deploying,
    Running,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    OpenClaw,
    Anthropic,
    OpenAI,
    BYOM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub provider: VpsProvider,
    pub region: Option<String>,
    pub status: DeploymentStatus,
    pub endpoint: Option<String>,
    pub gateway_url: Option<String>,
    pub volume_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VpsProvider {
    Railway,
    FlyIo,
    Aws,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Creating,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub master_id: Uuid,
    pub slave_ids: Vec<Uuid>,
    pub discord_channel_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub team_id: Uuid,
    pub assigned_to: Option<Uuid>,
    pub status: TaskStatus,
    pub description: String,
    pub result: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}
