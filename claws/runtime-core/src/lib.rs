use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
    OpenClaw,
    ZeroClaw,
    PicoClaw,
    NanoClaw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    OpenClaw,
    Anthropic,
    OpenAI,
    BYOM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordChannels {
    pub coordination_logs: String,
    pub slave_communication: String,
    pub master_orders: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAgent {
    pub id: String,
    pub name: String,
    pub discord_bot_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub discord_channels: Option<DiscordChannels>,
    pub model_provider: ModelProvider,
    pub model_api_key: Option<String>,
    pub model_endpoint: Option<String>,
    pub personality: Option<String>,
    pub skills: Vec<String>,
    pub workspace_dir: Option<String>,
    pub runtime_config: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub primary: RuntimeAgent,
    pub agents: Vec<RuntimeAgent>,
}

#[derive(Debug, Clone)]
pub struct RuntimeServicePort {
    pub port: u16,
    pub handlers: Vec<String>,
    pub internal_port: u16,
}

#[derive(Debug, Clone)]
pub struct RuntimePlan {
    pub env: BTreeMap<String, String>,
    pub init_script: String,
    pub services: Vec<RuntimeServicePort>,
}

pub trait ClawRuntime: Send + Sync {
    fn kind(&self) -> RuntimeKind;
    fn name(&self) -> &'static str;
    fn supports_multi_agent(&self) -> bool {
        false
    }
    fn build_plan(&self, ctx: &RuntimeContext) -> Result<RuntimePlan>;
}

pub fn runtime_name(kind: RuntimeKind) -> &'static str {
    match kind {
        RuntimeKind::OpenClaw => "openclaw",
        RuntimeKind::ZeroClaw => "zeroclaw",
        RuntimeKind::PicoClaw => "picoclaw",
        RuntimeKind::NanoClaw => "nanoclaw",
    }
}
