use axum::extract::{Path, State};
use axum::response::Json;
use engine::models::{AgentRole, AgentRuntime, AgentStatus, ModelProvider, VpsProvider};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::errors::AppError;
use crate::api::handlers::channels::TelegramSettings;
use crate::api::handlers::AppState;
use crate::api::services::agents::AgentService;

#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub role: AgentRole,
    pub provider: VpsProvider,
    pub railway_api_key: Option<String>,
    pub region: Option<String>,
    pub team_id: Option<Uuid>,
    pub discord_bot_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub runtime: Option<AgentRuntime>,
    pub model_provider: ModelProvider,
    pub model_api_key: Option<String>,
    pub model_endpoint: Option<String>,
    pub personality: Option<String>,
    pub skills: Vec<String>,
    pub runtime_config: Option<serde_json::Value>,
    pub responsibility: Option<String>,
    pub emoji: Option<String>,
}

#[derive(Deserialize)]
pub struct DeployMultiRequest {
    pub agent_ids: Vec<Uuid>,
    pub provider: VpsProvider,
    pub railway_api_key: Option<String>,
    pub region: Option<String>,
    pub telegram_settings: Option<TelegramSettings>,
}

#[derive(Serialize)]
pub struct AgentResponse {
    pub id: Uuid,
    pub name: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub runtime: AgentRuntime,
    pub responsibility: Option<String>,
    pub emoji: Option<String>,
}

pub async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<AgentResponse>, AppError> {
    let service = AgentService::new(&state);
    let response = service.create_agent(req).await?;
    Ok(Json(response))
}

pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<AgentResponse>>, AppError> {
    let service = AgentService::new(&state);
    let response = service.list_agents().await?;
    Ok(Json(response))
}

pub async fn get_agent_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentStatus>, AppError> {
    let service = AgentService::new(&state);
    let status = service.get_agent_status(id).await?;
    Ok(Json(status))
}

pub async fn deploy_agents_multi(
    State(state): State<AppState>,
    Json(req): Json<DeployMultiRequest>,
) -> Result<Json<crate::api::handlers::deployments::DeploymentResponse>, AppError> {
    let service = AgentService::new(&state);
    let deployment = service.deploy_agents_multi(req).await?;
    Ok(Json(deployment.into()))
}

pub async fn destroy_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let service = AgentService::new(&state);
    service.destroy_agent(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
