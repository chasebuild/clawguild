use anyhow::{Context, Result};
use claws_runtime_core::{
    ClawRuntime, DiscordChannels as RuntimeDiscordChannels, ModelProvider as RuntimeModelProvider,
    RuntimeAgent, RuntimeContext, RuntimeKind, RuntimePlan,
};
use nanoclaw_runtime::NanoClawRuntime;
use openclaw_runtime::OpenClawRuntime;
use picoclaw_runtime::PicoClawRuntime;
use std::sync::Arc;
use zeroclaw_runtime::ZeroClawRuntime;

use crate::models::{Agent, AgentRuntime, ModelProvider};

#[derive(Clone)]
pub struct RuntimeRegistry {
    openclaw: Arc<dyn ClawRuntime>,
    zeroclaw: Arc<dyn ClawRuntime>,
    picoclaw: Arc<dyn ClawRuntime>,
    nanoclaw: Arc<dyn ClawRuntime>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            openclaw: Arc::new(OpenClawRuntime::new()),
            zeroclaw: Arc::new(ZeroClawRuntime::new()),
            picoclaw: Arc::new(PicoClawRuntime::new()),
            nanoclaw: Arc::new(NanoClawRuntime::new()),
        }
    }

    pub fn get(&self, runtime: AgentRuntime) -> Arc<dyn ClawRuntime> {
        match runtime {
            AgentRuntime::OpenClaw => self.openclaw.clone(),
            AgentRuntime::ZeroClaw => self.zeroclaw.clone(),
            AgentRuntime::PicoClaw => self.picoclaw.clone(),
            AgentRuntime::NanoClaw => self.nanoclaw.clone(),
        }
    }

    pub fn build_plan(&self, agents: &[Agent]) -> Result<(RuntimeKind, RuntimePlan)> {
        let primary = agents
            .first()
            .context("at least one agent required to build runtime plan")?;
        let runtime = primary.runtime;

        for agent in agents.iter().skip(1) {
            if agent.runtime != runtime {
                anyhow::bail!("all agents in a multi-deploy must share the same runtime");
            }
        }

        let runtime_impl = self.get(runtime);
        if agents.len() > 1 && !runtime_impl.supports_multi_agent() {
            anyhow::bail!(
                "runtime {} does not support multi-agent deployments",
                runtime_impl.name()
            );
        }

        let primary_runtime = map_agent(primary)?;
        let agents_runtime = agents.iter().map(map_agent).collect::<Result<Vec<_>>>()?;

        let ctx = RuntimeContext {
            primary: primary_runtime,
            agents: agents_runtime,
        };

        Ok((runtime_impl.kind(), runtime_impl.build_plan(&ctx)?))
    }
}

fn map_agent(agent: &Agent) -> Result<RuntimeAgent> {
    Ok(RuntimeAgent {
        id: agent.id.to_string(),
        name: agent.name.clone(),
        discord_bot_token: agent.discord_bot_token.clone(),
        discord_channel_id: agent.discord_channel_id.clone(),
        discord_channels: agent.discord_channels.as_ref().map(map_discord_channels),
        model_provider: map_model_provider(agent.model_provider),
        model_api_key: agent.model_api_key.clone(),
        model_endpoint: agent.model_endpoint.clone(),
        personality: agent.personality.clone(),
        skills: agent.skills.clone(),
        workspace_dir: agent.workspace_dir.clone(),
        runtime_config: agent.runtime_config.clone(),
    })
}

fn map_discord_channels(channels: &crate::models::DiscordChannels) -> RuntimeDiscordChannels {
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
