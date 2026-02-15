pub mod handlers;
pub mod middleware;
pub mod routes;

use anyhow::Result;
use axum::Router;
use engine::coordinator::Coordinator;
use engine::deployment::manager::DeploymentManager;
use engine::storage::Database;
use std::time::Instant;

pub struct ApiServer {
    router: Router,
}

impl ApiServer {
    pub async fn new(
        db: Database,
        deployment_manager: DeploymentManager,
        coordinator: Coordinator,
        api_key: Option<String>,
        start_time: Instant,
    ) -> Result<Self> {
        let router =
            routes::create_router(db, deployment_manager, coordinator, api_key, start_time).await?;

        Ok(Self { router })
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        tracing::info!("API server listening on port {}", port);

        axum::serve(listener, self.router.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }
}
