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
    pub discord_channel_id: Option<String>, // Deprecated: use team's discord_channels
    pub discord_channels: Option<DiscordChannels>, // Agent-specific channel overrides (optional)
    pub model_provider: ModelProvider,
    pub model_api_key: Option<String>,
    pub model_endpoint: Option<String>,
    pub personality: Option<String>,
    pub skills: Vec<String>,
    pub workspace_dir: Option<String>,
    pub responsibility: Option<String>, // What the agent does (e.g., "Delegates, connects dots, ships")
    pub emoji: Option<String>,          // Emoji representing the agent's role (e.g., "🧰")
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
    /// Primary agent (first in multi-agent deployments); used for backward compatibility.
    pub agent_id: Uuid,
    /// When set, this VPS hosts multiple agents; coordination (Discord) is unchanged per agent.
    pub agent_ids: Option<Vec<Uuid>>,
    pub provider: VpsProvider,
    pub region: Option<String>,
    pub status: DeploymentStatus,
    /// Provider-specific ID (e.g. flyio-{machine_id}) for get_status/destroy.
    pub provider_id: Option<String>,
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
    pub discord_channel_id: String, // Main coordination channel (deprecated, use discord_channels)
    pub discord_channels: DiscordChannels,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordChannels {
    pub coordination_logs: String, // Channel for coordination logs and status updates
    pub slave_communication: String, // Channel for slave-to-slave and slave-to-master communication
    pub master_orders: String,     // Channel for master orders and task delegation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub team_id: Uuid,
    /// When set, this task is a subtask of another task.
    pub parent_task_id: Option<Uuid>,
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
