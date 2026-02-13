mod api;

use anyhow::Result;
use engine::{adapters, coordinator, deployment, Config, Database};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    eprintln!("Starting ClawGuild API Server");
    tracing::info!("Starting ClawGuild API Server");

    // Load configuration
    eprintln!("Loading configuration...");
    let config = Config::load()?;
    eprintln!("Database URL: {}", config.database_url);
    eprintln!("API Port: {}", config.api_port);

    // Initialize database
    eprintln!("Connecting to database...");
    let db = Database::new(&config.database_url).await?;
    eprintln!("Database connected successfully");
    eprintln!("Running migrations...");
    db.run_migrations().await?;
    eprintln!("Migrations completed");

    // Initialize VPS adapters
    eprintln!("Initializing VPS adapters...");
    let vps_adapters = adapters::VpsAdapters::new(&config).await?;
    eprintln!("VPS adapters initialized");

    // Initialize deployment manager
    eprintln!("Initializing deployment manager...");
    let deployment_manager =
        deployment::manager::DeploymentManager::new(db.clone(), vps_adapters).await?;
    eprintln!("Deployment manager initialized");

    // Initialize coordinator
    eprintln!("Initializing coordinator...");
    let coordinator =
        coordinator::Coordinator::new(db.clone(), config.discord_bot_token.clone()).await?;
    eprintln!("Coordinator initialized");

    // Initialize API server
    eprintln!("Initializing API server...");
    let start_time = std::time::Instant::now();
    let api_server = api::ApiServer::new(
        db.clone(),
        deployment_manager,
        coordinator,
        config.api_key.clone(),
        start_time,
    )
    .await?;
    eprintln!("API server initialized");

    // Start API server
    eprintln!("Starting API server on port {}...", config.api_port);
    api_server.start(config.api_port).await?;

    Ok(())
}
