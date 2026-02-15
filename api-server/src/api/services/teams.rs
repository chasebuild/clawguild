use crate::api::errors::AppError;
use crate::api::handlers::channels::apply_telegram_settings_to_agents_tx;
use crate::api::handlers::teams::{
    CreateTeamRequest, TeamResponse, TeamRosterMember, TeamRosterResponse,
};
use crate::api::handlers::AppState;
use engine::models::{AgentRole, Team};
use engine::storage::repositories::{AgentRepository, TeamRepository};
use uuid::Uuid;

pub struct TeamService<'a> {
    state: &'a AppState,
}

impl<'a> TeamService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn create_team(&self, req: CreateTeamRequest) -> Result<TeamResponse, AppError> {
        let discord_channels =
            req.discord_channels
                .unwrap_or_else(|| engine::models::DiscordChannels {
                    coordination_logs: req.discord_channel_id.clone(),
                    slave_communication: req.discord_channel_id.clone(),
                    master_orders: req.discord_channel_id.clone(),
                });

        let slave_ids: Vec<Uuid> = req
            .slave_ids
            .into_iter()
            .filter(|id| id != &req.master_id)
            .collect();

        let team = Team {
            id: Uuid::new_v4(),
            name: req.name,
            master_id: req.master_id,
            slave_ids,
            discord_channel_id: req.discord_channel_id,
            discord_channels,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let team_repo = TeamRepository::new(self.state.db.db().clone());
        let agent_repo = AgentRepository::new(self.state.db.db().clone());
        let mut tx = self
            .state
            .db
            .db()
            .begin()
            .await
            .map_err(|err| AppError::Internal(err.into()))?;

        team_repo
            .create_tx(&mut tx, &team)
            .await
            .map_err(AppError::Internal)?;

        agent_repo
            .update_role_tx(&mut tx, team.master_id, AgentRole::Master)
            .await
            .map_err(AppError::Internal)?;
        agent_repo
            .update_team_membership_tx(
                &mut tx,
                team.master_id,
                Some(team.id),
                Some(team.discord_channels.clone()),
                Some(team.discord_channel_id.clone()),
            )
            .await
            .map_err(AppError::Internal)?;

        for slave_id in &team.slave_ids {
            agent_repo
                .update_role_tx(&mut tx, *slave_id, AgentRole::Slave)
                .await
                .map_err(AppError::Internal)?;
            agent_repo
                .update_team_membership_tx(
                    &mut tx,
                    *slave_id,
                    Some(team.id),
                    Some(team.discord_channels.clone()),
                    Some(team.discord_channel_id.clone()),
                )
                .await
                .map_err(AppError::Internal)?;
        }

        if let Some(settings) = req.telegram_settings {
            let mut agent_ids = Vec::with_capacity(team.slave_ids.len() + 1);
            agent_ids.push(team.master_id);
            agent_ids.extend(team.slave_ids.iter().copied());
            let agents = fetch_agents(&agent_repo, &agent_ids).await?;
            apply_telegram_settings_to_agents_tx(&agent_repo, &mut tx, &settings, &agents).await?;
        }

        tx.commit()
            .await
            .map_err(|err| AppError::Internal(err.into()))?;

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            master_id: team.master_id,
            slave_ids: team.slave_ids,
            discord_channel_id: team.discord_channel_id,
        })
    }

    pub async fn list_teams(&self) -> Result<Vec<TeamResponse>, AppError> {
        let repo = TeamRepository::new(self.state.db.db().clone());
        let teams = repo.list_all().await.map_err(AppError::Internal)?;

        Ok(teams
            .into_iter()
            .map(|team| TeamResponse {
                id: team.id,
                name: team.name,
                master_id: team.master_id,
                slave_ids: team.slave_ids,
                discord_channel_id: team.discord_channel_id,
            })
            .collect())
    }

    pub async fn assign_agent_to_team(
        &self,
        team_id: Uuid,
        agent_id: Uuid,
        role: AgentRole,
    ) -> Result<TeamResponse, AppError> {
        let team_repo = TeamRepository::new(self.state.db.db().clone());
        let agent_repo = AgentRepository::new(self.state.db.db().clone());

        let team = team_repo
            .get_by_id(team_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("team not found".to_string()))?;
        let agent = agent_repo
            .get_by_id(agent_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("agent not found".to_string()))?;

        let mut slave_ids = team.slave_ids.clone();
        let master_id = if matches!(role, AgentRole::Master) {
            slave_ids.retain(|id| id != &agent.id);
            agent.id
        } else {
            if !slave_ids.contains(&agent.id) {
                slave_ids.push(agent.id);
            }
            team.master_id
        };

        let mut tx = self
            .state
            .db
            .db()
            .begin()
            .await
            .map_err(|err| AppError::Internal(err.into()))?;

        team_repo
            .update_members_tx(&mut tx, team.id, master_id, slave_ids.clone())
            .await
            .map_err(AppError::Internal)?;

        agent_repo
            .update_role_tx(&mut tx, agent.id, role)
            .await
            .map_err(AppError::Internal)?;
        agent_repo
            .update_team_membership_tx(
                &mut tx,
                agent.id,
                Some(team.id),
                Some(team.discord_channels.clone()),
                Some(team.discord_channel_id.clone()),
            )
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|err| AppError::Internal(err.into()))?;

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            master_id,
            slave_ids,
            discord_channel_id: team.discord_channel_id,
        })
    }

    pub async fn get_team_roster(&self, team_id: Uuid) -> Result<TeamRosterResponse, AppError> {
        let team_repo = TeamRepository::new(self.state.db.db().clone());
        let agent_repo = AgentRepository::new(self.state.db.db().clone());

        let team = team_repo
            .get_by_id(team_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("team not found".to_string()))?;

        let mut members = Vec::new();
        let mut all_ids = team.slave_ids.clone();
        all_ids.push(team.master_id);

        for agent_id in all_ids {
            if let Some(agent) = agent_repo
                .get_by_id(agent_id)
                .await
                .map_err(AppError::Internal)?
            {
                members.push(TeamRosterMember {
                    id: agent.id,
                    name: agent.name,
                    role: format!("{:?}", agent.role).to_lowercase(),
                    responsibility: agent
                        .responsibility
                        .unwrap_or_else(|| "unassigned".to_string()),
                    emoji: agent.emoji.unwrap_or_else(|| "🤖".to_string()),
                    status: agent.status,
                });
            }
        }

        Ok(TeamRosterResponse {
            team_id: team.id,
            team_name: team.name,
            members,
        })
    }
}

async fn fetch_agents(
    agent_repo: &AgentRepository,
    agent_ids: &[Uuid],
) -> Result<Vec<engine::models::Agent>, AppError> {
    let mut agents = Vec::with_capacity(agent_ids.len());
    for agent_id in agent_ids {
        let agent = agent_repo
            .get_by_id(*agent_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("agent not found".to_string()))?;
        agents.push(agent);
    }
    Ok(agents)
}
