use crate::api::errors::AppError;
use engine::models::{Agent, DiscordChannels, ModelProvider};
use openclaw_runtime::channel_adapters::apply_channel_adapters as apply_openclaw_channel_adapters;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

use claws_runtime_core::{
    DiscordChannels as RuntimeDiscordChannels, ModelProvider as RuntimeModelProvider, RuntimeAgent,
    RuntimeContext,
};

#[derive(Clone, Deserialize)]
pub struct TelegramSettings {
    pub enabled: Option<bool>,
    pub bot_token: Option<String>,
    pub dm_policy: Option<String>,
    pub allow_from: Option<Vec<String>>,
    pub group_policy: Option<String>,
    pub group_allow_from: Option<Vec<String>>,
    pub require_mention: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClawConfig {
    #[serde(default)]
    pub channels: OpenClawChannels,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClawChannels {
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelegramConfig {
    pub enabled: Option<bool>,
    pub bot_token: Option<String>,
    pub dm_policy: Option<String>,
    pub allow_from: Option<Vec<String>>,
    pub group_policy: Option<String>,
    pub group_allow_from: Option<Vec<String>>,
    pub groups: Option<HashMap<String, TelegramGroupConfig>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelegramGroupConfig {
    pub require_mention: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl OpenClawConfig {
    pub fn from_value(value: Option<Value>) -> Result<Self, AppError> {
        let value = value.unwrap_or_else(|| json!({}));
        serde_json::from_value(value)
            .map_err(|err| AppError::BadRequest(format!("invalid openclaw runtime_config: {err}")))
    }

    pub fn to_value(self) -> Result<Value, AppError> {
        serde_json::to_value(self).map_err(|err| AppError::Internal(err.into()))
    }

    pub fn apply_defaults(&mut self, defaults: OpenClawConfig) {
        self.channels.apply_defaults(defaults.channels);
    }

    pub fn apply_telegram_settings(&mut self, settings: &TelegramSettings) {
        let telegram = self
            .channels
            .telegram
            .get_or_insert_with(TelegramConfig::default);
        if let Some(enabled) = settings.enabled {
            telegram.enabled = Some(enabled);
        }
        if let Some(token) = &settings.bot_token {
            if !token.is_empty() {
                telegram.bot_token = Some(token.clone());
            }
        }
        if let Some(policy) = &settings.dm_policy {
            telegram.dm_policy = Some(policy.clone());
        }
        if let Some(list) = &settings.allow_from {
            if !list.is_empty() {
                telegram.allow_from = Some(list.clone());
            }
        }
        if let Some(policy) = &settings.group_policy {
            telegram.group_policy = Some(policy.clone());
        }
        if let Some(list) = &settings.group_allow_from {
            if !list.is_empty() {
                telegram.group_allow_from = Some(list.clone());
            }
        }
        if let Some(require_mention) = settings.require_mention {
            let groups = telegram.groups.get_or_insert_with(HashMap::new);
            let entry = groups
                .entry("*".to_string())
                .or_insert_with(TelegramGroupConfig::default);
            entry.require_mention = Some(require_mention);
        }
    }

    pub fn validate(&self) -> Result<(), AppError> {
        let Some(telegram) = &self.channels.telegram else {
            return Ok(());
        };
        let enabled = telegram.enabled.unwrap_or(false);
        let dm_policy = telegram.dm_policy.as_deref();
        if let Some(policy) = dm_policy {
            if !matches!(policy, "pairing" | "allowlist" | "open" | "disabled") {
                return Err(AppError::BadRequest("invalid dmPolicy".to_string()));
            }
        }
        let group_policy = telegram.group_policy.as_deref();
        if let Some(policy) = group_policy {
            if !matches!(policy, "open" | "allowlist" | "disabled") {
                return Err(AppError::BadRequest("invalid groupPolicy".to_string()));
            }
        }

        if enabled {
            let allow_from = telegram.allow_from.as_ref().map(Vec::as_slice);
            if let Some(policy) = dm_policy {
                match policy {
                    "open" => {
                        let has_star = allow_from
                            .map(|items| items.iter().any(|item| item == "*"))
                            .unwrap_or(false);
                        if !has_star {
                            return Err(AppError::BadRequest(
                                "dmPolicy open requires allowFrom to include '*'".to_string(),
                            ));
                        }
                    }
                    "allowlist" => {
                        let has_any = allow_from.map(|items| !items.is_empty()).unwrap_or(false);
                        if !has_any {
                            return Err(AppError::BadRequest(
                                "dmPolicy allowlist requires allowFrom".to_string(),
                            ));
                        }
                    }
                    _ => {}
                }
            }

            if let Some(policy) = group_policy {
                if policy == "allowlist" {
                    let has_any = telegram
                        .group_allow_from
                        .as_ref()
                        .map(|items| !items.is_empty())
                        .unwrap_or(false)
                        || allow_from.map(|items| !items.is_empty()).unwrap_or(false);
                    if !has_any {
                        return Err(AppError::BadRequest(
                            "groupPolicy allowlist requires groupAllowFrom or allowFrom"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl OpenClawChannels {
    fn apply_defaults(&mut self, defaults: OpenClawChannels) {
        match (&mut self.telegram, defaults.telegram) {
            (Some(existing), Some(defaults)) => existing.apply_defaults(defaults),
            (None, Some(defaults)) => {
                self.telegram = Some(defaults);
            }
            _ => {}
        }
    }
}

impl TelegramConfig {
    fn apply_defaults(&mut self, defaults: TelegramConfig) {
        if self.enabled.is_none() {
            self.enabled = defaults.enabled;
        }
        if self.bot_token.is_none() {
            self.bot_token = defaults.bot_token;
        }
        if self.dm_policy.is_none() {
            self.dm_policy = defaults.dm_policy;
        }
        if self.allow_from.is_none() {
            self.allow_from = defaults.allow_from;
        }
        if self.group_policy.is_none() {
            self.group_policy = defaults.group_policy;
        }
        if self.group_allow_from.is_none() {
            self.group_allow_from = defaults.group_allow_from;
        }
        if self.groups.is_none() {
            self.groups = defaults.groups;
        }
    }
}

pub fn openclaw_telegram_defaults_from_adapters(
    ctx: &RuntimeContext,
) -> Result<OpenClawConfig, AppError> {
    let defaults = apply_openclaw_channel_adapters(ctx);
    let telegram = defaults
        .get("channels")
        .and_then(|channels| channels.get("telegram"))
        .cloned();
    let value = match telegram {
        Some(telegram) => json!({ "channels": { "telegram": telegram } }),
        None => json!({}),
    };
    OpenClawConfig::from_value(Some(value))
}

pub fn openclaw_context_from_request(
    agent_id: Uuid,
    name: &str,
    discord_bot_token: Option<String>,
    discord_channel_id: Option<String>,
    discord_channels: Option<DiscordChannels>,
    model_provider: ModelProvider,
    model_api_key: Option<String>,
    model_endpoint: Option<String>,
    personality: Option<String>,
    skills: Vec<String>,
    runtime_config: Option<Value>,
) -> RuntimeContext {
    let runtime_agent = RuntimeAgent {
        id: agent_id.to_string(),
        name: name.to_string(),
        discord_bot_token,
        discord_channel_id,
        discord_channels: discord_channels.as_ref().map(map_discord_channels),
        model_provider: map_model_provider(model_provider),
        model_api_key,
        model_endpoint,
        personality,
        skills,
        workspace_dir: None,
        runtime_config,
    };

    RuntimeContext {
        primary: runtime_agent.clone(),
        agents: vec![runtime_agent],
    }
}

pub fn openclaw_context_from_agent(agent: &Agent) -> RuntimeContext {
    let runtime_agent = RuntimeAgent {
        id: agent.id.to_string(),
        name: agent.name.clone(),
        discord_bot_token: agent.discord_bot_token.clone(),
        discord_channel_id: agent.discord_channel_id.clone(),
        discord_channels: agent.discord_channels.as_ref().map(map_discord_channels),
        model_provider: map_model_provider(agent.model_provider.clone()),
        model_api_key: agent.model_api_key.clone(),
        model_endpoint: agent.model_endpoint.clone(),
        personality: agent.personality.clone(),
        skills: agent.skills.clone(),
        workspace_dir: agent.workspace_dir.clone(),
        runtime_config: agent.runtime_config.clone(),
    };

    RuntimeContext {
        primary: runtime_agent.clone(),
        agents: vec![runtime_agent],
    }
}

fn map_discord_channels(channels: &DiscordChannels) -> RuntimeDiscordChannels {
    RuntimeDiscordChannels {
        coordination_logs: channels.coordination_logs.clone(),
        slave_communication: channels.slave_communication.clone(),
        master_orders: channels.master_orders.clone(),
    }
}

fn map_model_provider(provider: ModelProvider) -> RuntimeModelProvider {
    match provider {
        ModelProvider::OpenClaw => RuntimeModelProvider::OpenClaw,
        ModelProvider::Anthropic => RuntimeModelProvider::Anthropic,
        ModelProvider::OpenAI => RuntimeModelProvider::OpenAI,
        ModelProvider::BYOM => RuntimeModelProvider::BYOM,
    }
}

pub async fn apply_telegram_settings_to_agents(
    agent_repo: &engine::storage::repositories::AgentRepository,
    settings: &TelegramSettings,
    agents: &[Agent],
) -> Result<(), AppError> {
    for agent in agents {
        if agent.runtime != engine::models::AgentRuntime::OpenClaw {
            continue;
        }
        let defaults =
            openclaw_telegram_defaults_from_adapters(&openclaw_context_from_agent(agent))?;
        let mut config = OpenClawConfig::from_value(agent.runtime_config.clone())?;
        config.apply_defaults(defaults);
        config.apply_telegram_settings(settings);
        config.validate()?;
        agent_repo
            .update_runtime_config(agent.id, Some(config.to_value()?))
            .await
            .map_err(|err| AppError::Internal(err.into()))?;
    }
    Ok(())
}

pub async fn apply_telegram_settings_to_agents_tx(
    agent_repo: &engine::storage::repositories::AgentRepository,
    tx: &mut Transaction<'_, Postgres>,
    settings: &TelegramSettings,
    agents: &[Agent],
) -> Result<(), AppError> {
    for agent in agents {
        if agent.runtime != engine::models::AgentRuntime::OpenClaw {
            continue;
        }
        let defaults =
            openclaw_telegram_defaults_from_adapters(&openclaw_context_from_agent(agent))?;
        let mut config = OpenClawConfig::from_value(agent.runtime_config.clone())?;
        config.apply_defaults(defaults);
        config.apply_telegram_settings(settings);
        config.validate()?;
        agent_repo
            .update_runtime_config_tx(tx, agent.id, Some(config.to_value()?))
            .await
            .map_err(|err| AppError::Internal(err.into()))?;
    }
    Ok(())
}
