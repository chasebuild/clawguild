use crate::models::{
    Agent, AgentRole, AgentRuntime, AgentStatus, Deployment, DeploymentStatus, DiscordChannels,
    ModelProvider, Task, TaskStatus, Team, VpsProvider,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(FromRow)]
struct AgentRow {
    id: Uuid,
    name: String,
    role: String,
    status: String,
    runtime: String,
    deployment_id: Option<Uuid>,
    team_id: Option<Uuid>,
    discord_bot_token: Option<String>,
    discord_channel_id: Option<String>,
    discord_channels: Option<Json<DiscordChannels>>,
    model_provider: String,
    model_api_key: Option<String>,
    model_endpoint: Option<String>,
    personality: Option<String>,
    skills: Vec<String>,
    workspace_dir: Option<String>,
    runtime_config: Option<Json<serde_json::Value>>,
    responsibility: Option<String>,
    emoji: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AgentRow> for Agent {
    type Error = anyhow::Error;

    fn try_from(row: AgentRow) -> Result<Self> {
        Ok(Agent {
            id: row.id,
            name: row.name,
            role: parse_agent_role(&row.role)?,
            status: parse_agent_status(&row.status)?,
            runtime: parse_agent_runtime(&row.runtime)?,
            deployment_id: row.deployment_id,
            team_id: row.team_id,
            discord_bot_token: row.discord_bot_token,
            discord_channel_id: row.discord_channel_id,
            discord_channels: row.discord_channels.map(|value| value.0),
            model_provider: parse_model_provider(&row.model_provider)?,
            model_api_key: row.model_api_key,
            model_endpoint: row.model_endpoint,
            personality: row.personality,
            skills: row.skills,
            workspace_dir: row.workspace_dir,
            runtime_config: row.runtime_config.map(|value| value.0),
            responsibility: row.responsibility,
            emoji: row.emoji,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(FromRow)]
struct DeploymentRow {
    id: Uuid,
    agent_id: Uuid,
    agent_ids: Option<Vec<Uuid>>,
    provider: String,
    region: Option<String>,
    status: String,
    provider_id: Option<String>,
    endpoint: Option<String>,
    gateway_url: Option<String>,
    volume_id: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<DeploymentRow> for Deployment {
    type Error = anyhow::Error;

    fn try_from(row: DeploymentRow) -> Result<Self> {
        Ok(Deployment {
            id: row.id,
            agent_id: row.agent_id,
            agent_ids: row.agent_ids,
            provider: parse_vps_provider(&row.provider)?,
            region: row.region,
            status: parse_deployment_status(&row.status)?,
            provider_id: row.provider_id,
            endpoint: row.endpoint,
            gateway_url: row.gateway_url,
            volume_id: row.volume_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(FromRow)]
struct TeamRow {
    id: Uuid,
    name: String,
    master_id: Uuid,
    slave_ids: Vec<Uuid>,
    discord_channel_id: String,
    discord_channels: Json<DiscordChannels>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TeamRow> for Team {
    fn from(row: TeamRow) -> Self {
        Team {
            id: row.id,
            name: row.name,
            master_id: row.master_id,
            slave_ids: row.slave_ids,
            discord_channel_id: row.discord_channel_id,
            discord_channels: row.discord_channels.0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(FromRow)]
struct TaskRow {
    id: Uuid,
    team_id: Uuid,
    parent_task_id: Option<Uuid>,
    assigned_to: Option<Uuid>,
    status: String,
    description: String,
    result: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TaskRow> for Task {
    type Error = anyhow::Error;

    fn try_from(row: TaskRow) -> Result<Self> {
        Ok(Task {
            id: row.id,
            team_id: row.team_id,
            parent_task_id: row.parent_task_id,
            assigned_to: row.assigned_to,
            status: parse_task_status(&row.status)?,
            description: row.description,
            result: row.result,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub struct AgentRepository {
    db: PgPool,
}

impl AgentRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, agent: &Agent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO agents (
                id, name, role, status, runtime, deployment_id, team_id, discord_bot_token,
                discord_channel_id, discord_channels, model_provider, model_api_key,
                model_endpoint, personality, skills, workspace_dir, runtime_config, responsibility, emoji,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18,
                $19, $20, $21
            )
            "#,
        )
        .bind(agent.id)
        .bind(&agent.name)
        .bind(agent_role_to_str(&agent.role))
        .bind(agent_status_to_str(&agent.status))
        .bind(agent_runtime_to_str(&agent.runtime))
        .bind(agent.deployment_id)
        .bind(agent.team_id)
        .bind(&agent.discord_bot_token)
        .bind(&agent.discord_channel_id)
        .bind(agent.discord_channels.clone().map(Json))
        .bind(model_provider_to_str(&agent.model_provider))
        .bind(&agent.model_api_key)
        .bind(&agent.model_endpoint)
        .bind(&agent.personality)
        .bind(&agent.skills)
        .bind(&agent.workspace_dir)
        .bind(agent.runtime_config.clone().map(Json))
        .bind(&agent.responsibility)
        .bind(&agent.emoji)
        .bind(agent.created_at)
        .bind(agent.updated_at)
        .execute(&self.db)
        .await
        .context("failed to create agent")?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Agent>> {
        let row: Option<AgentRow> = sqlx::query_as(
            r#"
            SELECT id, name, role, status, deployment_id, team_id, discord_bot_token,
                   discord_channel_id, discord_channels, model_provider, model_api_key,
                   model_endpoint, personality, skills, workspace_dir, runtime_config, responsibility, emoji,
                   runtime,
                   created_at, updated_at
            FROM agents
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        row.map(Agent::try_from).transpose()
    }

    pub async fn update_status(&self, id: Uuid, status: AgentStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET status = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(agent_status_to_str(&status))
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update agent status")?;

        Ok(())
    }

    pub async fn update_deployment_id(&self, id: Uuid, deployment_id: Option<Uuid>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET deployment_id = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(deployment_id)
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update agent deployment_id")?;

        Ok(())
    }

    pub async fn update_team_membership(
        &self,
        id: Uuid,
        team_id: Option<Uuid>,
        discord_channels: Option<DiscordChannels>,
        discord_channel_id: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET team_id = $2,
                discord_channels = $3,
                discord_channel_id = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(team_id)
        .bind(discord_channels.map(Json))
        .bind(discord_channel_id)
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update agent team membership")?;

        Ok(())
    }

    pub async fn update_team_membership_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        team_id: Option<Uuid>,
        discord_channels: Option<DiscordChannels>,
        discord_channel_id: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET team_id = $2,
                discord_channels = $3,
                discord_channel_id = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(team_id)
        .bind(discord_channels.map(Json))
        .bind(discord_channel_id)
        .bind(Utc::now())
        .execute(tx.as_mut())
        .await
        .context("failed to update agent team membership")?;

        Ok(())
    }

    pub async fn update_role(&self, id: Uuid, role: AgentRole) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET role = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(agent_role_to_str(&role))
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update agent role")?;

        Ok(())
    }

    pub async fn update_runtime_config(
        &self,
        id: Uuid,
        runtime_config: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET runtime_config = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(runtime_config.map(Json))
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update agent runtime_config")?;

        Ok(())
    }

    pub async fn update_role_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        role: AgentRole,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET role = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(agent_role_to_str(&role))
        .bind(Utc::now())
        .execute(tx.as_mut())
        .await
        .context("failed to update agent role")?;

        Ok(())
    }

    pub async fn update_runtime_config_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        runtime_config: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents
            SET runtime_config = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(runtime_config.map(Json))
        .bind(Utc::now())
        .execute(tx.as_mut())
        .await
        .context("failed to update agent runtime_config")?;

        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<Agent>> {
        let rows: Vec<AgentRow> = sqlx::query_as(
            r#"
            SELECT id, name, role, status, deployment_id, team_id, discord_bot_token,
                   discord_channel_id, discord_channels, model_provider, model_api_key,
                   model_endpoint, personality, skills, workspace_dir, runtime_config, responsibility, emoji,
                   runtime,
                   created_at, updated_at
            FROM agents
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        rows.into_iter().map(Agent::try_from).collect()
    }
}

pub struct DeploymentRepository {
    db: PgPool,
}

impl DeploymentRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, deployment: &Deployment) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO deployments (
                id, agent_id, agent_ids, provider, region, status, provider_id, endpoint,
                gateway_url, volume_id, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12
            )
            "#,
        )
        .bind(deployment.id)
        .bind(deployment.agent_id)
        .bind(&deployment.agent_ids)
        .bind(vps_provider_to_str(&deployment.provider))
        .bind(&deployment.region)
        .bind(deployment_status_to_str(&deployment.status))
        .bind(&deployment.provider_id)
        .bind(&deployment.endpoint)
        .bind(&deployment.gateway_url)
        .bind(&deployment.volume_id)
        .bind(deployment.created_at)
        .bind(deployment.updated_at)
        .execute(&self.db)
        .await
        .context("failed to create deployment")?;

        Ok(())
    }

    pub async fn update_status(&self, id: Uuid, status: DeploymentStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE deployments
            SET status = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(deployment_status_to_str(&status))
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update deployment status")?;

        Ok(())
    }

    pub async fn update_status_details(
        &self,
        id: Uuid,
        status: DeploymentStatus,
        endpoint: Option<String>,
        gateway_url: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE deployments
            SET status = $2,
                endpoint = $3,
                gateway_url = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(deployment_status_to_str(&status))
        .bind(endpoint)
        .bind(gateway_url)
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update deployment details")?;

        Ok(())
    }

    pub async fn update_provider_id(&self, id: Uuid, provider_id: String) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE deployments
            SET provider_id = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(provider_id)
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update deployment provider_id")?;

        Ok(())
    }

    pub async fn get_by_agent_id(&self, agent_id: Uuid) -> Result<Option<Deployment>> {
        let row: Option<DeploymentRow> = sqlx::query_as(
            r#"
            SELECT id, agent_id, agent_ids, provider, region, status, provider_id, endpoint,
                   gateway_url, volume_id, created_at, updated_at
            FROM deployments
            WHERE agent_id = $1 OR $1 = ANY(agent_ids)
            LIMIT 1
            "#,
        )
        .bind(agent_id)
        .fetch_optional(&self.db)
        .await?;

        row.map(Deployment::try_from).transpose()
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Deployment>> {
        let row: Option<DeploymentRow> = sqlx::query_as(
            r#"
            SELECT id, agent_id, agent_ids, provider, region, status, provider_id, endpoint,
                   gateway_url, volume_id, created_at, updated_at
            FROM deployments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        row.map(Deployment::try_from).transpose()
    }

    pub async fn list_all(&self) -> Result<Vec<Deployment>> {
        let rows: Vec<DeploymentRow> = sqlx::query_as(
            r#"
            SELECT id, agent_id, agent_ids, provider, region, status, provider_id, endpoint,
                   gateway_url, volume_id, created_at, updated_at
            FROM deployments
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        rows.into_iter().map(Deployment::try_from).collect()
    }
}

pub struct TeamRepository {
    db: PgPool,
}

impl TeamRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, team: &Team) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO teams (
                id, name, master_id, slave_ids, discord_channel_id, discord_channels, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(team.id)
        .bind(&team.name)
        .bind(team.master_id)
        .bind(&team.slave_ids)
        .bind(&team.discord_channel_id)
        .bind(Json(team.discord_channels.clone()))
        .bind(team.created_at)
        .bind(team.updated_at)
        .execute(&self.db)
        .await
        .context("failed to create team")?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Team>> {
        let row: Option<TeamRow> = sqlx::query_as(
            r#"
            SELECT id, name, master_id, slave_ids, discord_channel_id, discord_channels, created_at, updated_at
            FROM teams
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(Team::from))
    }

    pub async fn list_all(&self) -> Result<Vec<Team>> {
        let rows: Vec<TeamRow> = sqlx::query_as(
            r#"
            SELECT id, name, master_id, slave_ids, discord_channel_id, discord_channels, created_at, updated_at
            FROM teams
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(Team::from).collect())
    }

    pub async fn update_members(
        &self,
        id: Uuid,
        master_id: Uuid,
        slave_ids: Vec<Uuid>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE teams
            SET master_id = $2,
                slave_ids = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(master_id)
        .bind(slave_ids)
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update team members")?;

        Ok(())
    }

    pub async fn create_tx(&self, tx: &mut Transaction<'_, Postgres>, team: &Team) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO teams (
                id, name, master_id, slave_ids, discord_channel_id, discord_channels, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(team.id)
        .bind(&team.name)
        .bind(team.master_id)
        .bind(&team.slave_ids)
        .bind(&team.discord_channel_id)
        .bind(Json(team.discord_channels.clone()))
        .bind(team.created_at)
        .bind(team.updated_at)
        .execute(tx.as_mut())
        .await
        .context("failed to create team")?;

        Ok(())
    }

    pub async fn update_members_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        master_id: Uuid,
        slave_ids: Vec<Uuid>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE teams
            SET master_id = $2,
                slave_ids = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(master_id)
        .bind(slave_ids)
        .bind(Utc::now())
        .execute(tx.as_mut())
        .await
        .context("failed to update team members")?;

        Ok(())
    }
}

pub struct TaskRepository {
    db: PgPool,
}

impl TaskRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, task: &Task) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, team_id, parent_task_id, assigned_to, status, description, result, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(task.id)
        .bind(task.team_id)
        .bind(task.parent_task_id)
        .bind(task.assigned_to)
        .bind(task_status_to_str(&task.status))
        .bind(&task.description)
        .bind(&task.result)
        .bind(task.created_at)
        .bind(task.updated_at)
        .execute(&self.db)
        .await
        .context("failed to create task")?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Task>> {
        let row: Option<TaskRow> = sqlx::query_as(
            r#"
            SELECT id, team_id, parent_task_id, assigned_to, status, description, result, created_at, updated_at
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        row.map(Task::try_from).transpose()
    }

    pub async fn update_status(&self, id: Uuid, status: TaskStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE tasks
            SET status = $2,
                updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(task_status_to_str(&status))
        .bind(Utc::now())
        .execute(&self.db)
        .await
        .context("failed to update task status")?;

        Ok(())
    }

    pub async fn update_fields(
        &self,
        id: Uuid,
        status: Option<TaskStatus>,
        result: Option<String>,
    ) -> Result<Option<Task>> {
        let status_value = status.map(|value| task_status_to_str(&value).to_string());

        let row: Option<TaskRow> = sqlx::query_as(
            r#"
            UPDATE tasks
            SET status = COALESCE($2, status),
                result = COALESCE($3, result),
                updated_at = $4
            WHERE id = $1
            RETURNING id, team_id, parent_task_id, assigned_to, status, description, result, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(status_value)
        .bind(result)
        .bind(Utc::now())
        .fetch_optional(&self.db)
        .await?;

        row.map(Task::try_from).transpose()
    }

    pub async fn get_by_agent_id(&self, agent_id: Uuid) -> Result<Vec<Task>> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            r#"
            SELECT id, team_id, parent_task_id, assigned_to, status, description, result, created_at, updated_at
            FROM tasks
            WHERE assigned_to = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.db)
        .await?;

        rows.into_iter().map(Task::try_from).collect()
    }

    pub async fn get_by_parent_id(&self, parent_id: Uuid) -> Result<Vec<Task>> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            r#"
            SELECT id, team_id, parent_task_id, assigned_to, status, description, result, created_at, updated_at
            FROM tasks
            WHERE parent_task_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.db)
        .await?;

        rows.into_iter().map(Task::try_from).collect()
    }
}

fn parse_agent_role(value: &str) -> Result<AgentRole> {
    match value {
        "master" => Ok(AgentRole::Master),
        "slave" => Ok(AgentRole::Slave),
        _ => anyhow::bail!("invalid agent role: {}", value),
    }
}

fn parse_agent_status(value: &str) -> Result<AgentStatus> {
    match value {
        "pending" => Ok(AgentStatus::Pending),
        "deploying" => Ok(AgentStatus::Deploying),
        "running" => Ok(AgentStatus::Running),
        "stopped" => Ok(AgentStatus::Stopped),
        "error" => Ok(AgentStatus::Error),
        _ => anyhow::bail!("invalid agent status: {}", value),
    }
}

fn parse_agent_runtime(value: &str) -> Result<AgentRuntime> {
    match value {
        "openclaw" => Ok(AgentRuntime::OpenClaw),
        "zeroclaw" => Ok(AgentRuntime::ZeroClaw),
        "picoclaw" => Ok(AgentRuntime::PicoClaw),
        "nanoclaw" => Ok(AgentRuntime::NanoClaw),
        _ => anyhow::bail!("invalid agent runtime: {}", value),
    }
}

fn parse_model_provider(value: &str) -> Result<ModelProvider> {
    match value {
        "openclaw" => Ok(ModelProvider::OpenClaw),
        "anthropic" => Ok(ModelProvider::Anthropic),
        "openai" => Ok(ModelProvider::OpenAI),
        "byom" => Ok(ModelProvider::BYOM),
        _ => anyhow::bail!("invalid model provider: {}", value),
    }
}

fn parse_vps_provider(value: &str) -> Result<VpsProvider> {
    match value {
        "railway" => Ok(VpsProvider::Railway),
        "flyio" => Ok(VpsProvider::FlyIo),
        "aws" => Ok(VpsProvider::Aws),
        _ => anyhow::bail!("invalid vps provider: {}", value),
    }
}

fn parse_deployment_status(value: &str) -> Result<DeploymentStatus> {
    match value {
        "pending" => Ok(DeploymentStatus::Pending),
        "creating" => Ok(DeploymentStatus::Creating),
        "running" => Ok(DeploymentStatus::Running),
        "stopped" => Ok(DeploymentStatus::Stopped),
        "failed" => Ok(DeploymentStatus::Failed),
        _ => anyhow::bail!("invalid deployment status: {}", value),
    }
}

fn parse_task_status(value: &str) -> Result<TaskStatus> {
    match value {
        "pending" => Ok(TaskStatus::Pending),
        "in_progress" => Ok(TaskStatus::InProgress),
        "completed" => Ok(TaskStatus::Completed),
        "failed" => Ok(TaskStatus::Failed),
        _ => anyhow::bail!("invalid task status: {}", value),
    }
}

fn agent_role_to_str(role: &AgentRole) -> &'static str {
    match role {
        AgentRole::Master => "master",
        AgentRole::Slave => "slave",
    }
}

fn agent_status_to_str(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Pending => "pending",
        AgentStatus::Deploying => "deploying",
        AgentStatus::Running => "running",
        AgentStatus::Stopped => "stopped",
        AgentStatus::Error => "error",
    }
}

fn agent_runtime_to_str(runtime: &AgentRuntime) -> &'static str {
    match runtime {
        AgentRuntime::OpenClaw => "openclaw",
        AgentRuntime::ZeroClaw => "zeroclaw",
        AgentRuntime::PicoClaw => "picoclaw",
        AgentRuntime::NanoClaw => "nanoclaw",
    }
}

fn model_provider_to_str(provider: &ModelProvider) -> &'static str {
    match provider {
        ModelProvider::OpenClaw => "openclaw",
        ModelProvider::Anthropic => "anthropic",
        ModelProvider::OpenAI => "openai",
        ModelProvider::BYOM => "byom",
    }
}

fn vps_provider_to_str(provider: &VpsProvider) -> &'static str {
    match provider {
        VpsProvider::Railway => "railway",
        VpsProvider::FlyIo => "flyio",
        VpsProvider::Aws => "aws",
    }
}

fn deployment_status_to_str(status: &DeploymentStatus) -> &'static str {
    match status {
        DeploymentStatus::Pending => "pending",
        DeploymentStatus::Creating => "creating",
        DeploymentStatus::Running => "running",
        DeploymentStatus::Stopped => "stopped",
        DeploymentStatus::Failed => "failed",
    }
}

fn task_status_to_str(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Completed => "completed",
        TaskStatus::Failed => "failed",
    }
}
