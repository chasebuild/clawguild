use anyhow::Result;
use claws_runtime_core::{runtime_name, ClawRuntime, RuntimeContext, RuntimeKind, RuntimePlan};
use serde_json::Value;
use std::collections::BTreeMap;

mod channel_adapters;

use channel_adapters::apply_channel_adapters;
pub struct NanoClawRuntime;

impl Default for NanoClawRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl NanoClawRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl ClawRuntime for NanoClawRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::NanoClaw
    }

    fn name(&self) -> &'static str {
        runtime_name(self.kind())
    }

    fn build_plan(&self, ctx: &RuntimeContext) -> Result<RuntimePlan> {
        let settings = runtime_settings(&ctx.primary);

        let mut env = BTreeMap::new();
        if let Some(api_key) = settings.anthropic_api_key {
            env.insert("NANOCLAW_ANTHROPIC_API_KEY".to_string(), api_key);
        }
        if let Some(args) = settings.run_args {
            env.insert("NANOCLAW_RUN_ARGS".to_string(), args);
        }
        if let Some(repo_ref) = settings.repo_ref {
            env.insert("NANOCLAW_REF".to_string(), repo_ref);
        }

        let init_script = include_str!("../scripts/init.sh").to_string();

        Ok(RuntimePlan {
            env,
            init_script,
            services: Vec::new(),
        })
    }
}

struct NanoClawSettings {
    anthropic_api_key: Option<String>,
    run_args: Option<String>,
    repo_ref: Option<String>,
}

fn runtime_settings(agent: &claws_runtime_core::RuntimeAgent) -> NanoClawSettings {
    let mut anthropic_api_key = agent.model_api_key.clone();
    let mut run_args = None;
    let mut repo_ref = None;

    let mut merged_config = apply_channel_adapters(agent);
    if let Some(runtime_config) = &agent.runtime_config {
        merge_json(&mut merged_config, runtime_config);
    }

    if let Some(map) = merged_config.as_object() {
        if let Some(Value::String(value)) = map.get("anthropic_api_key") {
            anthropic_api_key = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("run_args") {
            run_args = Some(value.clone());
        }
        if let Some(Value::String(value)) = map.get("repo_ref") {
            repo_ref = Some(value.clone());
        }
    }

    NanoClawSettings {
        anthropic_api_key,
        run_args,
        repo_ref,
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
