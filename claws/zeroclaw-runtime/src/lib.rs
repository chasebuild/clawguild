use anyhow::Result;
use claws_runtime_core::{
    runtime_name, ClawRuntime, ModelProvider, RuntimeContext, RuntimeKind, RuntimePlan,
};
use serde_json::Value;
use std::collections::BTreeMap;

mod channel_adapters;

use channel_adapters::apply_channel_adapters;
pub struct ZeroClawRuntime;

impl Default for ZeroClawRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroClawRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl ClawRuntime for ZeroClawRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::ZeroClaw
    }

    fn name(&self) -> &'static str {
        runtime_name(self.kind())
    }

    fn build_plan(&self, ctx: &RuntimeContext) -> Result<RuntimePlan> {
        let settings = runtime_settings(&ctx.primary);

        let mut env = BTreeMap::new();
        if let Some(provider) = settings.model_provider {
            env.insert("ZEROCLAW_MODEL_PROVIDER".to_string(), provider);
        }
        if let Some(model) = settings.model {
            env.insert("ZEROCLAW_MODEL".to_string(), model);
        }
        if let Some(api_key) = settings.api_key {
            env.insert("ZEROCLAW_API_KEY".to_string(), api_key);
        }
        if let Some(token) = settings.discord_token {
            env.insert("ZEROCLAW_DISCORD_TOKEN".to_string(), token);
        }

        let init_script = include_str!("../scripts/init.sh").to_string();

        Ok(RuntimePlan {
            env,
            init_script,
            services: Vec::new(),
        })
    }
}

struct ZeroClawSettings {
    model_provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    discord_token: Option<String>,
}

fn runtime_settings(agent: &claws_runtime_core::RuntimeAgent) -> ZeroClawSettings {
    let mut model_provider = match agent.model_provider {
        ModelProvider::OpenAI => Some("openai".to_string()),
        ModelProvider::Anthropic => Some("anthropic".to_string()),
        ModelProvider::BYOM => Some("openrouter".to_string()),
        ModelProvider::OpenClaw => None,
    };
    let mut model = match agent.model_provider {
        ModelProvider::OpenAI => Some("gpt-4o-mini".to_string()),
        ModelProvider::Anthropic => Some("claude-3-5-sonnet-20240620".to_string()),
        ModelProvider::BYOM => Some("openrouter/auto".to_string()),
        ModelProvider::OpenClaw => None,
    };
    let mut api_key = agent.model_api_key.clone();
    let mut discord_token = agent.discord_bot_token.clone();

    let mut merged_config = apply_channel_adapters(agent);
    if let Some(runtime_config) = &agent.runtime_config {
        merge_json(&mut merged_config, runtime_config);
    }

    if let Some(map) = merged_config.as_object() {
        if let Some(Value::String(value)) = map.get("model_provider") {
            model_provider = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("model") {
            model = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("api_key") {
            api_key = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("discord_token") {
            discord_token = Some(value.clone());
        }
    }

    ZeroClawSettings {
        model_provider,
        model,
        api_key,
        discord_token,
    }
}

fn merge_json(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                merge_json(base_map.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (base_slot, overlay_val) => {
            *base_slot = overlay_val.clone();
        }
    }
}
