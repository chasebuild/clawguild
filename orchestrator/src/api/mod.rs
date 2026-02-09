pub mod routes;
pub mod handlers;
pub mod middleware;

use anyhow::Result;
use axum::Router;
use crate::deployment::manager::DeploymentManager;
use crate::coordinator::Coordinator;
use crate::storage::Database;

pub struct ApiServer {
    router: Router,
}

impl ApiServer {
    pub async fn new(
        db: Database,
        deployment_manager: DeploymentManager,
        coordinator: Coordinator,
    ) -> Result<Self> {
        let router = routes::create_router(db, deployment_manager, coordinator).await?;
        
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
