use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use engine::coordinator::Coordinator;
use engine::deployment::manager::DeploymentManager;
use engine::models::{
    Agent, AgentRole, AgentRuntime, AgentStatus, DiscordChannels, ModelProvider, Task, TaskStatus,
    Team, VpsProvider,
};
use engine::storage::Database;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub deployment_manager: DeploymentManager,
    #[allow(dead_code)]
    pub coordinator: Coordinator,
    pub api_key: Option<String>,
    pub start_time: Instant,
}

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub master_id: Uuid,
    pub slave_ids: Vec<Uuid>,
    pub discord_channel_id: String, // Legacy: single channel
    pub discord_channels: Option<DiscordChannels>, // New: multiple channels
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
    // Use provided discord_channels or create from single channel_id (legacy)
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

    let repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());
    repo.create(&team)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let agent_repo = engine::storage::repositories::AgentRepository::new(state.db.db().clone());
    agent_repo
        .update_role(team.master_id, AgentRole::Master)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    agent_repo
        .update_team_membership(
            team.master_id,
            Some(team.id),
            Some(team.discord_channels.clone()),
            Some(team.discord_channel_id.clone()),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for slave_id in &team.slave_ids {
        agent_repo
            .update_role(*slave_id, AgentRole::Slave)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        agent_repo
            .update_team_membership(
                *slave_id,
                Some(team.id),
                Some(team.discord_channels.clone()),
                Some(team.discord_channel_id.clone()),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

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
    let repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());
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
    pub region: Option<String>,
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
) -> Result<Json<AgentResponse>, StatusCode> {
    let mut discord_channels = None;
    let mut discord_channel_id = req.discord_channel_id.clone();
    if let Some(team_id) = req.team_id {
        let team_repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());
        let team = team_repo
            .get_by_id(team_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        discord_channels = Some(team.discord_channels);
        if discord_channel_id.is_none() {
            discord_channel_id = Some(team.discord_channel_id);
        }
    }

    let agent = Agent {
        id: Uuid::new_v4(),
        name: req.name,
        role: req.role,
        status: AgentStatus::Pending,
        runtime: req.runtime.unwrap_or(AgentRuntime::OpenClaw),
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
        runtime_config: req.runtime_config,
        responsibility: req.responsibility,
        emoji: req.emoji,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let agent_repo = engine::storage::repositories::AgentRepository::new(state.db.db().clone());
    agent_repo
        .create(&agent)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(team_id) = agent.team_id {
        let team_repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());
        let team = team_repo
            .get_by_id(team_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

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
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

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
        runtime: agent.runtime,
        responsibility: agent.responsibility,
        emoji: agent.emoji,
    }))
}

pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<AgentResponse>>, StatusCode> {
    let repo = engine::storage::repositories::AgentRepository::new(state.db.db().clone());
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
            runtime: agent.runtime,
            responsibility: agent.responsibility,
            emoji: agent.emoji,
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

pub async fn deploy_agents_multi(
    State(state): State<AppState>,
    Json(req): Json<DeployMultiRequest>,
) -> Result<Json<DeploymentResponse>, StatusCode> {
    if req.agent_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let agent_repo = engine::storage::repositories::AgentRepository::new(state.db.db().clone());
    let mut agents = Vec::with_capacity(req.agent_ids.len());
    for id in &req.agent_ids {
        let agent = agent_repo
            .get_by_id(*id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        agents.push(agent);
    }

    let deployment = state
        .deployment_manager
        .deploy_agents_multi(agents, req.provider, req.region)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DeploymentResponse {
        id: deployment.id,
        agent_id: deployment.agent_id,
        agent_ids: deployment.agent_ids,
        provider: format!("{:?}", deployment.provider),
        region: deployment.region,
        status: format!("{:?}", deployment.status),
        endpoint: deployment.endpoint,
        gateway_url: deployment.gateway_url,
        created_at: deployment.created_at,
        updated_at: deployment.updated_at,
    }))
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
    let agent_repo = engine::storage::repositories::AgentRepository::new(state.db.db().clone());
    let agent = agent_repo
        .get_by_id(agent_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let team_id = agent.team_id.ok_or(StatusCode::BAD_REQUEST)?;

    // Create task
    let mut task = Task {
        id: Uuid::new_v4(),
        team_id,
        parent_task_id: None,
        assigned_to: Some(agent_id),
        status: TaskStatus::Pending,
        description: req.description,
        result: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Save task
    let task_repo = engine::storage::repositories::TaskRepository::new(state.db.db().clone());
    task_repo
        .create(&task)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Use coordinator to delegate task if agent is master
    if matches!(agent.role, AgentRole::Master) {
        let team_repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());
        let team = team_repo
            .get_by_id(team_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let _subtasks = state
            .coordinator
            .master()
            .delegate_task(&team, &task)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let task_repo = engine::storage::repositories::TaskRepository::new(state.db.db().clone());
        task_repo
            .update_fields(task.id, Some(TaskStatus::InProgress), None)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        task.status = TaskStatus::InProgress;
    }

    Ok(Json(task))
}

pub async fn get_agent_tasks(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let task_repo = engine::storage::repositories::TaskRepository::new(state.db.db().clone());
    let tasks = task_repo
        .get_by_agent_id(agent_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tasks))
}

#[derive(Deserialize)]
pub struct AssignAgentRequest {
    pub agent_id: Uuid,
    pub role: AgentRole,
}

pub async fn assign_agent_to_team(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
    Json(req): Json<AssignAgentRequest>,
) -> Result<Json<TeamResponse>, StatusCode> {
    let team_repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());
    let agent_repo = engine::storage::repositories::AgentRepository::new(state.db.db().clone());

    let team = team_repo
        .get_by_id(team_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let agent = agent_repo
        .get_by_id(req.agent_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut slave_ids = team.slave_ids.clone();
    let master_id = if matches!(req.role, AgentRole::Master) {
        slave_ids.retain(|id| id != &agent.id);
        agent.id
    } else {
        if !slave_ids.contains(&agent.id) {
            slave_ids.push(agent.id);
        }
        team.master_id
    };

    team_repo
        .update_members(team.id, master_id, slave_ids.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    agent_repo
        .update_role(agent.id, req.role)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    agent_repo
        .update_team_membership(
            agent.id,
            Some(team.id),
            Some(team.discord_channels.clone()),
            Some(team.discord_channel_id.clone()),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TeamResponse {
        id: team.id,
        name: team.name,
        master_id,
        slave_ids,
        discord_channel_id: team.discord_channel_id,
    }))
}

#[derive(Serialize)]
pub struct TeamRosterMember {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub responsibility: String,
    pub emoji: String,
    pub status: AgentStatus,
}

#[derive(Serialize)]
pub struct TeamRosterResponse {
    pub team_id: Uuid,
    pub team_name: String,
    pub members: Vec<TeamRosterMember>,
}

pub async fn get_team_roster(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> Result<Json<TeamRosterResponse>, StatusCode> {
    // Get team
    let team_repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());
    let team = team_repo
        .get_by_id(team_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get all agents in the team
    let agent_repo = engine::storage::repositories::AgentRepository::new(state.db.db().clone());

    let mut members = Vec::new();

    // Get master agent
    if let Ok(Some(master)) = agent_repo.get_by_id(team.master_id).await {
        // Determine role title based on personality or default
        let role_title = if let Some(ref personality) = master.personality {
            personality.clone()
        } else {
            "CEO".to_string() // Default for master
        };

        members.push(TeamRosterMember {
            id: master.id,
            name: master.name.clone(),
            role: role_title,
            responsibility: master
                .responsibility
                .clone()
                .unwrap_or_else(|| "Delegates, connects dots, ships".to_string()),
            emoji: master.emoji.clone().unwrap_or_else(|| "🧰".to_string()),
            status: master.status,
        });
    }

    // Get slave agents
    for slave_id in &team.slave_ids {
        if let Ok(Some(slave)) = agent_repo.get_by_id(*slave_id).await {
            // Determine role title based on personality or skills
            let role_title = if let Some(ref personality) = slave.personality {
                personality.clone()
            } else if !slave.skills.is_empty() {
                slave.skills[0].clone()
            } else {
                "Specialist".to_string()
            };

            members.push(TeamRosterMember {
                id: slave.id,
                name: slave.name.clone(),
                role: role_title,
                responsibility: slave
                    .responsibility
                    .clone()
                    .unwrap_or_else(|| "Executes assigned tasks".to_string()),
                emoji: slave.emoji.clone().unwrap_or_else(|| "🤖".to_string()),
                status: slave.status,
            });
        }
    }

    Ok(Json(TeamRosterResponse {
        team_id: team.id,
        team_name: team.name,
        members,
    }))
}

#[derive(Serialize)]
pub struct ServerHealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
}

pub async fn get_server_health_with_state(
    State(state): State<AppState>,
) -> Result<Json<ServerHealthResponse>, StatusCode> {
    Ok(Json(ServerHealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    }))
}

#[derive(Serialize)]
pub struct ServerStatusResponse {
    pub status: String,
    pub version: String,
    pub database_connected: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub async fn get_server_status(
    State(state): State<AppState>,
) -> Result<Json<ServerStatusResponse>, StatusCode> {
    // Check database connection by attempting a simple query
    let db_connected = sqlx::query("SELECT 1")
        .execute(&state.db.db())
        .await
        .is_ok();

    Ok(Json(ServerStatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database_connected: db_connected,
        timestamp: chrono::Utc::now(),
    }))
}

#[derive(Serialize)]
pub struct DeploymentResponse {
    pub id: Uuid,
    pub agent_id: Uuid,
    /// When set, this VPS hosts multiple agents (multi-agent deploy).
    pub agent_ids: Option<Vec<Uuid>>,
    pub provider: String,
    pub region: Option<String>,
    pub status: String,
    pub endpoint: Option<String>,
    pub gateway_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_deployments(
    State(state): State<AppState>,
) -> Result<Json<Vec<DeploymentResponse>>, StatusCode> {
    let repo = engine::storage::repositories::DeploymentRepository::new(state.db.db().clone());
    let deployments = repo
        .list_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<DeploymentResponse> = deployments
        .into_iter()
        .map(|d| DeploymentResponse {
            id: d.id,
            agent_id: d.agent_id,
            agent_ids: d.agent_ids,
            provider: format!("{:?}", d.provider),
            region: d.region,
            status: format!("{:?}", d.status),
            endpoint: d.endpoint,
            gateway_url: d.gateway_url,
            created_at: d.created_at,
            updated_at: d.updated_at,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_deployment(
    State(state): State<AppState>,
    Path(deployment_id): Path<Uuid>,
) -> Result<Json<DeploymentResponse>, StatusCode> {
    let repo = engine::storage::repositories::DeploymentRepository::new(state.db.db().clone());
    let deployment = repo
        .get_by_id(deployment_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(DeploymentResponse {
        id: deployment.id,
        agent_id: deployment.agent_id,
        agent_ids: deployment.agent_ids,
        provider: format!("{:?}", deployment.provider),
        region: deployment.region,
        status: format!("{:?}", deployment.status),
        endpoint: deployment.endpoint,
        gateway_url: deployment.gateway_url,
        created_at: deployment.created_at,
        updated_at: deployment.updated_at,
    }))
}

pub async fn get_deployment_logs(
    State(state): State<AppState>,
    Path(deployment_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let repo = engine::storage::repositories::DeploymentRepository::new(state.db.db().clone());
    let deployment = repo
        .get_by_id(deployment_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get lines parameter
    let lines = params
        .get("lines")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    // Get VPS provider adapter
    let vps_provider = state
        .deployment_manager
        .vps_adapters
        .get_provider(deployment.provider.clone())
        .ok_or(StatusCode::NOT_FOUND)?;

    let provider_id = deployment
        .provider_id
        .unwrap_or_else(|| match deployment.provider {
            engine::models::VpsProvider::FlyIo => format!("flyio-{}", deployment.id),
            engine::models::VpsProvider::Railway => format!("railway-{}", deployment.id),
            engine::models::VpsProvider::Aws => format!("aws-{}", deployment.id),
        });

    let deployment_id_struct = engine::adapters::trait_def::DeploymentId {
        id: deployment.id,
        provider_id,
    };

    // Get logs from VPS provider
    let logs: Vec<String> = vps_provider
        .get_logs(&deployment_id_struct, Some(lines))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(logs))
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<TaskStatus>,
    pub result: Option<String>,
}

pub async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, StatusCode> {
    if req.status.is_none() && req.result.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let task_repo = engine::storage::repositories::TaskRepository::new(state.db.db().clone());
    let updated = task_repo
        .update_fields(task_id, req.status, req.result)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated = updated.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(updated))
}

#[derive(Serialize)]
pub struct TaskAggregateResponse {
    pub task_id: Uuid,
    pub aggregated_result: String,
}

pub async fn aggregate_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskAggregateResponse>, StatusCode> {
    let task_repo = engine::storage::repositories::TaskRepository::new(state.db.db().clone());
    let team_repo = engine::storage::repositories::TeamRepository::new(state.db.db().clone());

    let task = task_repo
        .get_by_id(task_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let team = team_repo
        .get_by_id(task.team_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let aggregated = state
        .coordinator
        .master()
        .aggregate_results(&task, &team)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TaskAggregateResponse {
        task_id,
        aggregated_result: aggregated,
    }))
}
