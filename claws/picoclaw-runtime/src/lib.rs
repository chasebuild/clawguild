use anyhow::Result;
use claws_runtime_core::{runtime_name, ClawRuntime, RuntimeContext, RuntimeKind, RuntimePlan};
use serde_json::Value;
use std::collections::BTreeMap;

mod channel_adapters;

use channel_adapters::apply_channel_adapters;
pub struct PicoClawRuntime;

impl Default for PicoClawRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl PicoClawRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl ClawRuntime for PicoClawRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::PicoClaw
    }

    fn name(&self) -> &'static str {
        runtime_name(self.kind())
    }

    fn build_plan(&self, ctx: &RuntimeContext) -> Result<RuntimePlan> {
        let settings = runtime_settings(&ctx.primary);

        let mut env = BTreeMap::new();
        if let Some(api_key) = settings.openrouter_api_key {
            env.insert("PICOCLAW_OPENROUTER_API_KEY".to_string(), api_key);
        }
        if let Some(token) = settings.discord_token {
            env.insert("PICOCLAW_DISCORD_TOKEN".to_string(), token);
        }

        let init_script = include_str!("../scripts/init.sh").to_string();

        Ok(RuntimePlan {
            env,
            init_script,
            services: Vec::new(),
        })
    }
}

struct PicoClawSettings {
    openrouter_api_key: Option<String>,
    discord_token: Option<String>,
}

fn runtime_settings(agent: &claws_runtime_core::RuntimeAgent) -> PicoClawSettings {
    let mut openrouter_api_key = agent.model_api_key.clone();
    let mut discord_token = agent.discord_bot_token.clone();

    let mut merged_config = apply_channel_adapters(agent);
    if let Some(runtime_config) = &agent.runtime_config {
        merge_json(&mut merged_config, runtime_config);
    }

    if let Some(map) = merged_config.as_object() {
        if let Some(Value::String(value)) = map.get("openrouter_api_key") {
            openrouter_api_key = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("discord_token") {
            discord_token = Some(value.clone());
        }
    }

    PicoClawSettings {
        openrouter_api_key,
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
