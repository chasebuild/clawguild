use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        eprintln!("Attempting to connect to database at: {}", database_url);

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
            .with_context(|| format!("failed to connect to postgres at {}", database_url))?;

        Ok(Self { pool })
    }

    pub fn db(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS agents (
                id uuid PRIMARY KEY,
                name text NOT NULL,
                role text NOT NULL,
                status text NOT NULL,
                runtime text NOT NULL DEFAULT 'openclaw',
                deployment_id uuid,
                team_id uuid,
                discord_bot_token text,
                discord_channel_id text,
                discord_channels jsonb,
                model_provider text NOT NULL,
                model_api_key text,
                model_endpoint text,
                personality text,
                skills text[] NOT NULL DEFAULT '{}',
                workspace_dir text,
                runtime_config jsonb,
                responsibility text,
                emoji text,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL
            );
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            ALTER TABLE agents
            ADD COLUMN IF NOT EXISTS runtime text NOT NULL DEFAULT 'openclaw',
            ADD COLUMN IF NOT EXISTS runtime_config jsonb
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deployments (
                id uuid PRIMARY KEY,
                agent_id uuid NOT NULL,
                agent_ids uuid[],
                provider text NOT NULL,
                region text,
                status text NOT NULL,
                provider_id text,
                endpoint text,
                gateway_url text,
                volume_id text,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL
            );
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS teams (
                id uuid PRIMARY KEY,
                name text NOT NULL,
                master_id uuid NOT NULL,
                slave_ids uuid[] NOT NULL DEFAULT '{}',
                discord_channel_id text NOT NULL,
                discord_channels jsonb NOT NULL,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL
            );
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id uuid PRIMARY KEY,
                team_id uuid NOT NULL,
                parent_task_id uuid,
                assigned_to uuid,
                status text NOT NULL,
                description text NOT NULL,
                result text,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL
            );
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_agents_team_id ON agents(team_id);")
            .execute(&mut *tx)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_agents_deployment_id ON agents(deployment_id);")
            .execute(&mut *tx)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deployments_agent_id ON deployments(agent_id);")
            .execute(&mut *tx)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_team_id ON tasks(team_id);")
            .execute(&mut *tx)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_assigned_to ON tasks(assigned_to);")
            .execute(&mut *tx)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_parent_id ON tasks(parent_task_id);")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
