use crate::coordinator::Coordinator;
use crate::deployment::manager::DeploymentManager;
use crate::models::{Agent, AgentRole, AgentStatus, ModelProvider, Task, Team, VpsProvider};
use crate::storage::Database;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub deployment_manager: DeploymentManager,
    #[allow(dead_code)]
    pub coordinator: Coordinator,
}

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub master_id: Uuid,
    pub slave_ids: Vec<Uuid>,
    pub discord_channel_id: String,
}

#[derive(Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub master_id: Uuid,
    pub slave_ids: Vec<Uuid>,
    pub discord_channel_id: String,
}

pub async fn create_team(
    State(state): State<AppState>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<TeamResponse>, StatusCode> {
    let team = Team {
        id: Uuid::new_v4(),
        name: req.name,
        master_id: req.master_id,
        slave_ids: req.slave_ids,
        discord_channel_id: req.discord_channel_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let repo = crate::storage::repositories::TeamRepository::new(state.db.db().clone());
    repo.create(&team)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TeamResponse {
        id: team.id,
        name: team.name,
        master_id: team.master_id,
        slave_ids: team.slave_ids,
        discord_channel_id: team.discord_channel_id,
    }))
}

pub async fn list_teams(
    State(state): State<AppState>,
) -> Result<Json<Vec<TeamResponse>>, StatusCode> {
    let repo = crate::storage::repositories::TeamRepository::new(state.db.db().clone());
    let teams = repo
        .list_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<TeamResponse> = teams
        .into_iter()
        .map(|team| TeamResponse {
            id: team.id,
            name: team.name,
            master_id: team.master_id,
            slave_ids: team.slave_ids,
            discord_channel_id: team.discord_channel_id,
        })
        .collect();

    Ok(Json(responses))
}

#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub role: AgentRole,
    pub provider: VpsProvider,
    pub region: Option<String>,
    pub discord_bot_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub model_provider: ModelProvider,
    pub model_api_key: Option<String>,
    pub model_endpoint: Option<String>,
    pub personality: Option<String>,
    pub skills: Vec<String>,
}

#[derive(Serialize)]
pub struct AgentResponse {
    pub id: Uuid,
    pub name: String,
    pub role: AgentRole,
    pub status: AgentStatus,
}

pub async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<AgentResponse>, StatusCode> {
    let agent = Agent {
        id: Uuid::new_v4(),
        name: req.name,
        role: req.role,
        status: AgentStatus::Pending,
        deployment_id: None,
        team_id: None,
        discord_bot_token: req.discord_bot_token,
        discord_channel_id: req.discord_channel_id,
        model_provider: req.model_provider,
        model_api_key: req.model_api_key,
        model_endpoint: req.model_endpoint,
        personality: req.personality,
        skills: req.skills,
        workspace_dir: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let agent_repo = crate::storage::repositories::AgentRepository::new(state.db.db().clone());
    agent_repo
        .create(&agent)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Deploy agent
    let _deployment = state
        .deployment_manager
        .deploy_agent(agent.clone(), req.provider, req.region)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AgentResponse {
        id: agent.id,
        name: agent.name,
        role: agent.role,
        status: agent.status,
    }))
}

pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<AgentResponse>>, StatusCode> {
    let repo = crate::storage::repositories::AgentRepository::new(state.db.db().clone());
    let agents = repo
        .list_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<AgentResponse> = agents
        .into_iter()
        .map(|agent| AgentResponse {
            id: agent.id,
            name: agent.name,
            role: agent.role,
            status: agent.status,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_agent_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentStatus>, StatusCode> {
    let status = state
        .deployment_manager
        .get_agent_status(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(status))
}

pub async fn destroy_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .deployment_manager
        .destroy_agent(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct SendTaskRequest {
    pub description: String,
}

pub async fn send_task(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<SendTaskRequest>,
) -> Result<Json<Task>, StatusCode> {
    // Get agent to find team
    let agent_repo = crate::storage::repositories::AgentRepository::new(state.db.db().clone());
    let agent = agent_repo
        .get_by_id(agent_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let team_id = agent.team_id.ok_or(StatusCode::BAD_REQUEST)?;

    // Create task
    let task = Task {
        id: Uuid::new_v4(),
        team_id,
        assigned_to: Some(agent_id),
        status: crate::models::TaskStatus::Pending,
        description: req.description,
        result: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Save task
    let task_repo = crate::storage::repositories::TaskRepository::new(state.db.db().clone());
    task_repo
        .create(&task)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Use coordinator to delegate task if agent is master
    if matches!(agent.role, AgentRole::Master) {
        let team_repo = crate::storage::repositories::TeamRepository::new(state.db.db().clone());
        let team = team_repo
            .get_by_id(team_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let master_coord = crate::coordinator::master::MasterCoordinator;
        let _delegated_task = master_coord
            .delegate_task(&team, &task.description)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(task))
}

pub async fn get_agent_tasks(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let task_repo = crate::storage::repositories::TaskRepository::new(state.db.db().clone());
    let tasks = task_repo
        .get_by_agent_id(agent_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tasks))
}
