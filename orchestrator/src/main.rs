mod adapters;
mod api;
mod config;
mod coordinator;
mod deployment;
mod models;
mod storage;

use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting ClawGuild Orchestrator");

    // Load configuration
    let config = config::Config::load()?;

    // Initialize database
    let db = storage::database::Database::new(&config.database_url).await?;
    db.run_migrations().await?;

    // Initialize VPS adapters
    let vps_adapters = adapters::VpsAdapters::new(&config).await?;

    // Initialize deployment manager
    let deployment_manager =
        deployment::manager::DeploymentManager::new(db.clone(), vps_adapters).await?;

    // Initialize coordinator
    let coordinator =
        coordinator::Coordinator::new(db.clone(), config.discord_bot_token.clone()).await?;

    // Initialize API server
    let api_server = api::ApiServer::new(db.clone(), deployment_manager, coordinator).await?;

    // Start API server
    api_server.start(config.api_port).await?;

    Ok(())
}
