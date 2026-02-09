use anyhow::Result;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::Surreal;

#[derive(Clone)]
pub struct Database {
    db: Arc<Surreal<Client>>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Parse connection string format: ws://localhost:8000 or http://localhost:8000
        let db = Surreal::new::<Ws>(database_url).await?;

        // Sign in (if credentials are provided in the URL or use default)
        // For now, we'll use root credentials or no auth
        // db.signin(surrealdb::opt::auth::Root {
        //     username: "root",
        //     password: "root",
        // }).await?;

        // Use namespace and database
        db.use_ns("clawguild").use_db("clawguild").await?;

        Ok(Self { db: Arc::new(db) })
    }

    pub fn db(&self) -> Surreal<Client> {
        (*self.db).clone()
    }

    pub async fn run_migrations(&self) -> Result<()> {
        // SurrealDB doesn't need explicit table creation, but we can define schemas
        // For now, we'll just ensure the database is ready
        // SurrealDB will create records automatically

        // Define schemas for better type safety (optional)
        let _ = self.db.query(
            r#"
            DEFINE TABLE agents SCHEMAFULL;
            DEFINE FIELD id ON agents TYPE record(agents) | string;
            DEFINE FIELD name ON agents TYPE string;
            DEFINE FIELD role ON agents TYPE string ASSERT $value IN ['master', 'slave'];
            DEFINE FIELD status ON agents TYPE string ASSERT $value IN ['pending', 'deploying', 'running', 'stopped', 'error'];
            DEFINE FIELD deployment_id ON agents TYPE option<record(deployments) | string>;
            DEFINE FIELD team_id ON agents TYPE option<record(teams) | string>;
                DEFINE FIELD discord_bot_token ON agents TYPE option<string>;
                DEFINE FIELD discord_channel_id ON agents TYPE option<string>;
                DEFINE FIELD discord_channels ON agents TYPE option<object>;
                DEFINE FIELD discord_channels.coordination_logs ON agents TYPE option<string>;
                DEFINE FIELD discord_channels.slave_communication ON agents TYPE option<string>;
                DEFINE FIELD discord_channels.master_orders ON agents TYPE option<string>;
            DEFINE FIELD model_provider ON agents TYPE string ASSERT $value IN ['openclaw', 'anthropic', 'openai', 'byom'];
            DEFINE FIELD model_api_key ON agents TYPE option<string>;
            DEFINE FIELD model_endpoint ON agents TYPE option<string>;
            DEFINE FIELD personality ON agents TYPE option<string>;
            DEFINE FIELD skills ON agents TYPE array<string>;
            DEFINE FIELD workspace_dir ON agents TYPE option<string>;
            DEFINE FIELD created_at ON agents TYPE datetime;
            DEFINE FIELD updated_at ON agents TYPE datetime;
            DEFINE INDEX idx_agents_team_id ON agents FIELDS team_id;
            DEFINE INDEX idx_agents_deployment_id ON agents FIELDS deployment_id;
            "#
        ).await?;

        let _ = self.db.query(
            r#"
            DEFINE TABLE deployments SCHEMAFULL;
            DEFINE FIELD id ON deployments TYPE record(deployments) | string;
            DEFINE FIELD agent_id ON deployments TYPE record(agents) | string;
            DEFINE FIELD provider ON deployments TYPE string ASSERT $value IN ['railway', 'flyio', 'aws'];
            DEFINE FIELD region ON deployments TYPE option<string>;
            DEFINE FIELD status ON deployments TYPE string ASSERT $value IN ['pending', 'creating', 'running', 'stopped', 'failed'];
            DEFINE FIELD endpoint ON deployments TYPE option<string>;
            DEFINE FIELD gateway_url ON deployments TYPE option<string>;
            DEFINE FIELD volume_id ON deployments TYPE option<string>;
            DEFINE FIELD created_at ON deployments TYPE datetime;
            DEFINE FIELD updated_at ON deployments TYPE datetime;
            DEFINE INDEX idx_deployments_agent_id ON deployments FIELDS agent_id;
            "#
        ).await?;

        let _ = self
            .db
            .query(
                r#"
            DEFINE TABLE teams SCHEMAFULL;
            DEFINE FIELD id ON teams TYPE record(teams) | string;
            DEFINE FIELD name ON teams TYPE string;
            DEFINE FIELD master_id ON teams TYPE record(agents) | string;
            DEFINE FIELD slave_ids ON teams TYPE array<record(agents) | string>;
            DEFINE FIELD discord_channel_id ON teams TYPE string;
            DEFINE FIELD discord_channels ON teams TYPE object;
            DEFINE FIELD discord_channels.coordination_logs ON teams TYPE string;
            DEFINE FIELD discord_channels.slave_communication ON teams TYPE string;
            DEFINE FIELD discord_channels.master_orders ON teams TYPE string;
            DEFINE FIELD created_at ON teams TYPE datetime;
            DEFINE FIELD updated_at ON teams TYPE datetime;
            "#,
            )
            .await?;

        let _ = self.db.query(
            r#"
            DEFINE TABLE tasks SCHEMAFULL;
            DEFINE FIELD id ON tasks TYPE record(tasks) | string;
            DEFINE FIELD team_id ON tasks TYPE record(teams) | string;
            DEFINE FIELD assigned_to ON tasks TYPE option<record(agents) | string>;
            DEFINE FIELD status ON tasks TYPE string ASSERT $value IN ['pending', 'in_progress', 'completed', 'failed'];
            DEFINE FIELD description ON tasks TYPE string;
            DEFINE FIELD result ON tasks TYPE option<string>;
            DEFINE FIELD created_at ON tasks TYPE datetime;
            DEFINE FIELD updated_at ON tasks TYPE datetime;
            DEFINE INDEX idx_tasks_team_id ON tasks FIELDS team_id;
            DEFINE INDEX idx_tasks_assigned_to ON tasks FIELDS assigned_to;
            "#
        ).await?;

        Ok(())
    }
}
