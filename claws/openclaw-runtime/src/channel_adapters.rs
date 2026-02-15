use claws_runtime_core::RuntimeContext;
use serde_json::{json, Value};

pub trait ChannelAdapter {
    fn apply(&self, ctx: &RuntimeContext) -> Value;
}

pub struct DiscordChannelAdapter;

impl ChannelAdapter for DiscordChannelAdapter {
    fn apply(&self, ctx: &RuntimeContext) -> Value {
        let agents = if ctx.agents.is_empty() {
            vec![ctx.primary.clone()]
        } else {
            ctx.agents.clone()
        };

        let mut bindings = Vec::new();
        let mut bot_token: Option<String> = None;

        for agent in &agents {
            if let Some(token) = &agent.discord_bot_token {
                bot_token.get_or_insert_with(|| token.clone());
            }

            if let Some(channels) = &agent.discord_channels {
                bindings.push(channel_binding(
                    &channels.coordination_logs,
                    &agent.id,
                    "coordination_logs",
                ));
                bindings.push(channel_binding(
                    &channels.slave_communication,
                    &agent.id,
                    "slave_communication",
                ));
                bindings.push(channel_binding(
                    &channels.master_orders,
                    &agent.id,
                    "master_orders",
                ));
            } else if let Some(channel_id) = &agent.discord_channel_id {
                bindings.push(json!({
                    "channelId": channel_id,
                    "agentId": agent.id,
                }));
            }
        }

        if bot_token.is_none() || bindings.is_empty() {
            return json!({});
        }

        json!({
            "channels": {
                "discord": {
                    "token": bot_token,
                    "bindings": bindings,
                }
            }
        })
    }
}

pub struct TelegramChannelAdapter;

impl ChannelAdapter for TelegramChannelAdapter {
    fn apply(&self, _ctx: &RuntimeContext) -> Value {
        json!({
            "channels": {
                "telegram": {
                    "enabled": false,
                    "dmPolicy": "pairing",
                    "groupPolicy": "allowlist",
                    "groups": { "*": { "requireMention": true } }
                }
            }
        })
    }
}

pub fn apply_channel_adapters(ctx: &RuntimeContext) -> Value {
    let adapters: [&dyn ChannelAdapter; 2] = [&DiscordChannelAdapter, &TelegramChannelAdapter];
    let mut merged = json!({});
    for adapter in adapters {
        merge_json(&mut merged, adapter.apply(ctx));
    }
    merged
}

fn channel_binding(channel_id: &str, agent_id: &str, purpose: &str) -> Value {
    json!({
        "channelId": channel_id,
        "agentId": agent_id,
        "purpose": purpose,
    })
}

fn merge_json(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                merge_json(base_map.entry(key).or_insert(Value::Null), value);
            }
        }
        (base_slot, overlay_val) => {
            *base_slot = overlay_val;
        }
    }
}
