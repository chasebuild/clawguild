mod api;

use anyhow::Result;
use engine::{adapters, coordinator, deployment, Config, Database};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting ClawGuild API server");

    // Load configuration
    tracing::info!("loading configuration");
    let config = Config::load()?;
    tracing::info!(database_url = %redact_database_url(&config.database_url), "database configured");
    tracing::info!(api_port = config.api_port, "api port configured");

    // Initialize database
    tracing::info!("connecting to database");
    let db = Database::new(&config.database_url).await?;
    tracing::info!("database connected");
    tracing::info!("running migrations");
    db.run_migrations().await?;
    tracing::info!("migrations completed");

    // Initialize VPS adapters
    tracing::info!("initializing VPS adapters");
    let vps_adapters = adapters::VpsAdapters::new(&config).await?;
    tracing::info!("VPS adapters initialized");

    // Initialize deployment manager
    tracing::info!("initializing deployment manager");
    let deployment_manager =
        deployment::manager::DeploymentManager::new(db.clone(), vps_adapters).await?;
    tracing::info!("deployment manager initialized");

    // Initialize coordinator
    tracing::info!("initializing coordinator");
    let coordinator =
        coordinator::Coordinator::new(db.clone(), config.discord_bot_token.clone()).await?;
    tracing::info!("coordinator initialized");

    // Initialize API server
    tracing::info!("initializing API server");
    let start_time = std::time::Instant::now();
    let api_server = api::ApiServer::new(
        db.clone(),
        deployment_manager,
        coordinator,
        config.api_key.clone(),
        start_time,
    )
    .await?;
    tracing::info!("API server initialized");

    // Start API server
    tracing::info!(api_port = config.api_port, "starting API server");
    api_server.start(config.api_port).await?;

    Ok(())
}

fn redact_database_url(url: &str) -> String {
    let scheme_end = match url.find("://") {
        Some(index) => index + 3,
        None => return url.to_string(),
    };

    let at_index = match url[scheme_end..].find('@') {
        Some(index) => scheme_end + index,
        None => return url.to_string(),
    };

    let credentials = &url[scheme_end..at_index];
    let colon_index = match credentials.find(':') {
        Some(index) => scheme_end + index,
        None => return url.to_string(),
    };

    let mut redacted = String::with_capacity(url.len());
    redacted.push_str(&url[..colon_index + 1]);
    redacted.push_str("***");
    redacted.push_str(&url[at_index..]);
    redacted
}
