use crate::models::{Agent, AgentStatus, Deployment, DeploymentStatus, Task, TaskStatus, Team};
use anyhow::Result;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use uuid::Uuid;

pub struct AgentRepository {
    db: Surreal<Client>,
}

impl AgentRepository {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn create(&self, agent: &Agent) -> Result<()> {
        let record: Option<Agent> = self
            .db
            .create(("agents", agent.id.to_string()))
            .content(agent)
            .await?;

        if record.is_none() {
            anyhow::bail!("Failed to create agent");
        }

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Agent>> {
        let agent: Option<Agent> = self.db.select(("agents", id.to_string())).await?;

        Ok(agent)
    }

    pub async fn update_status(&self, id: Uuid, status: AgentStatus) -> Result<()> {
        let _: Option<Agent> = self
            .db
            .update(("agents", id.to_string()))
            .merge(serde_json::json!({
                "status": status,
                "updated_at": chrono::Utc::now(),
            }))
            .await?;

        Ok(())
    }

    pub async fn update_deployment_id(&self, id: Uuid, deployment_id: Option<Uuid>) -> Result<()> {
        let _: Option<Agent> = self
            .db
            .update(("agents", id.to_string()))
            .merge(serde_json::json!({
                "deployment_id": deployment_id.map(|u| u.to_string()),
                "updated_at": chrono::Utc::now(),
            }))
            .await?;

        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<Agent>> {
        let agents: Vec<Agent> = self.db.select("agents").await?;

        Ok(agents)
    }
}

pub struct DeploymentRepository {
    db: Surreal<Client>,
}

impl DeploymentRepository {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn create(&self, deployment: &Deployment) -> Result<()> {
        let record: Option<Deployment> = self
            .db
            .create(("deployments", deployment.id.to_string()))
            .content(deployment)
            .await?;

        if record.is_none() {
            anyhow::bail!("Failed to create deployment");
        }

        Ok(())
    }

    pub async fn update_status(&self, id: Uuid, status: DeploymentStatus) -> Result<()> {
        let _: Option<Deployment> = self
            .db
            .update(("deployments", id.to_string()))
            .merge(serde_json::json!({
                "status": status,
                "updated_at": chrono::Utc::now(),
            }))
            .await?;

        Ok(())
    }

    /// Find deployment by primary agent_id or by membership in agent_ids (multi-agent VPS).
    pub async fn get_by_agent_id(&self, agent_id: Uuid) -> Result<Option<Deployment>> {
        let aid = agent_id.to_string();
        let mut result = self
            .db
            .query(
                "SELECT * FROM deployments WHERE agent_id = $aid OR (agent_ids != NONE AND agent_ids CONTAINS $aid) LIMIT 1",
            )
            .bind(("aid", aid))
            .await?;

        let deployment: Option<Deployment> = result.take(0)?;
        Ok(deployment)
    }

    pub async fn update_provider_id(&self, id: Uuid, provider_id: String) -> Result<()> {
        let _: Option<Deployment> = self
            .db
            .update(("deployments", id.to_string()))
            .merge(serde_json::json!({
                "provider_id": provider_id,
                "updated_at": chrono::Utc::now(),
            }))
            .await?;

        Ok(())
    }
}

pub struct TeamRepository {
    db: Surreal<Client>,
}

impl TeamRepository {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn create(&self, team: &Team) -> Result<()> {
        let record: Option<Team> = self
            .db
            .create(("teams", team.id.to_string()))
            .content(team)
            .await?;

        if record.is_none() {
            anyhow::bail!("Failed to create team");
        }

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Team>> {
        let team: Option<Team> = self.db.select(("teams", id.to_string())).await?;

        Ok(team)
    }

    pub async fn list_all(&self) -> Result<Vec<Team>> {
        let teams: Vec<Team> = self.db.select("teams").await?;

        Ok(teams)
    }
}

pub struct TaskRepository {
    db: Surreal<Client>,
}

impl TaskRepository {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn create(&self, task: &Task) -> Result<()> {
        let record: Option<Task> = self
            .db
            .create(("tasks", task.id.to_string()))
            .content(task)
            .await?;

        if record.is_none() {
            anyhow::bail!("Failed to create task");
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn update_status(&self, id: Uuid, status: TaskStatus) -> Result<()> {
        let _: Option<Task> = self
            .db
            .update(("tasks", id.to_string()))
            .merge(serde_json::json!({
                "status": status,
                "updated_at": chrono::Utc::now(),
            }))
            .await?;

        Ok(())
    }

    pub async fn get_by_agent_id(&self, agent_id: Uuid) -> Result<Vec<Task>> {
        let mut result = self
            .db
            .query("SELECT * FROM tasks WHERE assigned_to = $agent_id")
            .bind(("agent_id", agent_id.to_string()))
            .await?;

        let tasks: Vec<Task> = result.take(0)?;
        Ok(tasks)
    }
}
