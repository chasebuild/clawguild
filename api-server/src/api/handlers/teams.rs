use axum::extract::{Path, State};
use axum::response::Json;
use engine::models::{AgentRole, DiscordChannels};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::errors::AppError;
use crate::api::handlers::channels::TelegramSettings;
use crate::api::handlers::AppState;
use crate::api::services::teams::TeamService;

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub master_id: Uuid,
    pub slave_ids: Vec<Uuid>,
    pub discord_channel_id: String, // Legacy: single channel
    pub discord_channels: Option<DiscordChannels>, // New: multiple channels
    pub telegram_settings: Option<TelegramSettings>,
}

#[derive(Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub master_id: Uuid,
    pub slave_ids: Vec<Uuid>,
    pub discord_channel_id: String,
}

#[derive(Deserialize)]
pub struct AssignAgentRequest {
    pub agent_id: Uuid,
    pub role: AgentRole,
}

#[derive(Serialize)]
pub struct TeamRosterMember {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub responsibility: String,
    pub emoji: String,
    pub status: engine::models::AgentStatus,
}

#[derive(Serialize)]
pub struct TeamRosterResponse {
    pub team_id: Uuid,
    pub team_name: String,
    pub members: Vec<TeamRosterMember>,
}

pub async fn create_team(
    State(state): State<AppState>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<TeamResponse>, AppError> {
    let service = TeamService::new(&state);
    let response = service.create_team(req).await?;
    Ok(Json(response))
}

pub async fn list_teams(
    State(state): State<AppState>,
) -> Result<Json<Vec<TeamResponse>>, AppError> {
    let service = TeamService::new(&state);
    let response = service.list_teams().await?;
    Ok(Json(response))
}

pub async fn assign_agent_to_team(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
    Json(req): Json<AssignAgentRequest>,
) -> Result<Json<TeamResponse>, AppError> {
    let service = TeamService::new(&state);
    let response = service
        .assign_agent_to_team(team_id, req.agent_id, req.role)
        .await?;
    Ok(Json(response))
}

pub async fn get_team_roster(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> Result<Json<TeamRosterResponse>, AppError> {
    let service = TeamService::new(&state);
    let response = service.get_team_roster(team_id).await?;
    Ok(Json(response))
}
