use claws_runtime_core::RuntimeAgent;
use serde_json::{json, Value};

pub trait ChannelAdapter {
    fn apply(&self, _agent: &RuntimeAgent) -> Value;
}

pub struct NoopChannelAdapter;

impl ChannelAdapter for NoopChannelAdapter {
    fn apply(&self, _agent: &RuntimeAgent) -> Value {
        json!({})
    }
}

pub fn apply_channel_adapters(agent: &RuntimeAgent) -> Value {
    let adapters: [&dyn ChannelAdapter; 1] = [&NoopChannelAdapter];
    let mut merged = json!({});
    for adapter in adapters {
        merge_json(&mut merged, adapter.apply(agent));
    }
    merged
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
