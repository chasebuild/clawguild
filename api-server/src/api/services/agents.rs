use crate::api::errors::AppError;
use crate::api::handlers::agents::{AgentResponse, CreateAgentRequest, DeployMultiRequest};
use crate::api::handlers::channels::{apply_telegram_settings_to_agents, TelegramSettings};
use crate::api::handlers::channels::{
    openclaw_context_from_agent, openclaw_context_from_request,
    openclaw_telegram_defaults_from_adapters, OpenClawConfig,
};
use crate::api::handlers::AppState;
use engine::models::{Agent, AgentRole, AgentRuntime, AgentStatus};
use engine::storage::repositories::{AgentRepository, TeamRepository};
use uuid::Uuid;

pub struct AgentService<'a> {
    state: &'a AppState,
}

impl<'a> AgentService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn create_agent(&self, req: CreateAgentRequest) -> Result<AgentResponse, AppError> {
        let mut discord_channels = None;
        let mut discord_channel_id = req.discord_channel_id.clone();
        if let Some(team_id) = req.team_id {
            let team_repo = TeamRepository::new(self.state.db.db().clone());
            let team = team_repo
                .get_by_id(team_id)
                .await
                .map_err(AppError::Internal)?
                .ok_or_else(|| AppError::NotFound("team not found".to_string()))?;
            discord_channels = Some(team.discord_channels);
            if discord_channel_id.is_none() {
                discord_channel_id = Some(team.discord_channel_id);
            }
        }

        let runtime = req.runtime.unwrap_or(AgentRuntime::OpenClaw);
        let agent_id = Uuid::new_v4();
        let runtime_config = match runtime {
            AgentRuntime::OpenClaw => {
                let ctx = openclaw_context_from_request(
                    agent_id,
                    &req.name,
                    req.discord_bot_token.clone(),
                    discord_channel_id.clone(),
                    discord_channels.clone(),
                    req.model_provider.clone(),
                    req.model_api_key.clone(),
                    req.model_endpoint.clone(),
                    req.personality.clone(),
                    req.skills.clone(),
                    req.runtime_config.clone(),
                );
                let defaults = openclaw_telegram_defaults_from_adapters(&ctx)?;
                let mut config = OpenClawConfig::from_value(req.runtime_config.clone())?;
                config.apply_defaults(defaults);
                config.validate()?;
                Some(config.to_value()?)
            }
            _ => req.runtime_config.clone(),
        };

        let agent = Agent {
            id: agent_id,
            name: req.name,
            role: req.role,
            status: AgentStatus::Pending,
            runtime,
            deployment_id: None,
            team_id: req.team_id,
            discord_bot_token: req.discord_bot_token,
            discord_channel_id,
            discord_channels,
            model_provider: req.model_provider,
            model_api_key: req.model_api_key,
            model_endpoint: req.model_endpoint,
            personality: req.personality,
            skills: req.skills,
            workspace_dir: None,
            runtime_config,
            responsibility: req.responsibility,
            emoji: req.emoji,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let agent_repo = AgentRepository::new(self.state.db.db().clone());
        agent_repo
            .create(&agent)
            .await
            .map_err(AppError::Internal)?;

        if let Some(team_id) = agent.team_id {
            let team_repo = TeamRepository::new(self.state.db.db().clone());
            let team = team_repo
                .get_by_id(team_id)
                .await
                .map_err(AppError::Internal)?
                .ok_or_else(|| AppError::NotFound("team not found".to_string()))?;

            let mut slave_ids = team.slave_ids.clone();
            let master_id = if matches!(agent.role, AgentRole::Master) {
                agent.id
            } else {
                if !slave_ids.contains(&agent.id) {
                    slave_ids.push(agent.id);
                }
                team.master_id
            };

            team_repo
                .update_members(team.id, master_id, slave_ids)
                .await
                .map_err(AppError::Internal)?;
        }

        self.state
            .deployment_manager
            .deploy_agent(agent.clone(), req.provider, req.region)
            .await
            .map_err(AppError::Internal)?;

        Ok(AgentResponse {
            id: agent.id,
            name: agent.name,
            role: agent.role,
            status: agent.status,
            runtime: agent.runtime,
            responsibility: agent.responsibility,
            emoji: agent.emoji,
        })
    }

    pub async fn list_agents(&self) -> Result<Vec<AgentResponse>, AppError> {
        let repo = AgentRepository::new(self.state.db.db().clone());
        let agents = repo.list_all().await.map_err(AppError::Internal)?;

        Ok(agents
            .into_iter()
            .map(|agent| AgentResponse {
                id: agent.id,
                name: agent.name,
                role: agent.role,
                status: agent.status,
                runtime: agent.runtime,
                responsibility: agent.responsibility,
                emoji: agent.emoji,
            })
            .collect())
    }

    pub async fn get_agent_status(&self, id: Uuid) -> Result<AgentStatus, AppError> {
        let status = self
            .state
            .deployment_manager
            .get_agent_status(id)
            .await
            .map_err(AppError::Internal)?;
        Ok(status)
    }

    pub async fn destroy_agent(&self, id: Uuid) -> Result<(), AppError> {
        self.state
            .deployment_manager
            .destroy_agent(id)
            .await
            .map_err(AppError::Internal)?;
        Ok(())
    }

    pub async fn deploy_agents_multi(
        &self,
        req: DeployMultiRequest,
    ) -> Result<engine::models::Deployment, AppError> {
        if req.agent_ids.is_empty() {
            return Err(AppError::BadRequest(
                "agent_ids cannot be empty".to_string(),
            ));
        }

        let agent_repo = AgentRepository::new(self.state.db.db().clone());
        let mut agents = Vec::with_capacity(req.agent_ids.len());
        for id in &req.agent_ids {
            let agent = agent_repo
                .get_by_id(*id)
                .await
                .map_err(AppError::Internal)?
                .ok_or_else(|| AppError::NotFound("agent not found".to_string()))?;
            agents.push(agent);
        }

        if let Some(settings) = req.telegram_settings.clone() {
            apply_telegram_settings_to_agents(&agent_repo, &settings, &agents).await?;
            for agent in &mut agents {
                if agent.runtime != AgentRuntime::OpenClaw {
                    continue;
                }
                let config = apply_openclaw_telegram_settings(
                    agent.runtime_config.clone(),
                    &settings,
                    agent,
                )?;
                agent.runtime_config = Some(config);
            }
        }

        let deployment = self
            .state
            .deployment_manager
            .deploy_agents_multi(agents, req.provider, req.region)
            .await
            .map_err(AppError::Internal)?;

        Ok(deployment)
    }
}

fn apply_openclaw_telegram_settings(
    runtime_config: Option<serde_json::Value>,
    settings: &TelegramSettings,
    agent: &Agent,
) -> Result<serde_json::Value, AppError> {
    let defaults = openclaw_telegram_defaults_from_adapters(&openclaw_context_from_agent(agent))?;
    let mut config = OpenClawConfig::from_value(runtime_config)?;
    config.apply_defaults(defaults);
    config.apply_telegram_settings(settings);
    config.validate()?;
    config.to_value()
}
