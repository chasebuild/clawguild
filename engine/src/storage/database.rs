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
        sqlx::migrate!("../infra/migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }
}
