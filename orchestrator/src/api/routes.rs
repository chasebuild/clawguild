use axum::Router;
use tower_http::cors::CorsLayer;
use crate::deployment::manager::DeploymentManager;
use crate::coordinator::Coordinator;
use crate::storage::Database;
use crate::api::handlers;

pub async fn create_router(
    db: Database,
    deployment_manager: DeploymentManager,
    coordinator: Coordinator,
) -> anyhow::Result<Router> {
    let router = Router::new()
        .route("/api/teams", axum::routing::post(handlers::create_team))
        .route("/api/teams", axum::routing::get(handlers::list_teams))
        .route("/api/agents", axum::routing::post(handlers::create_agent))
        .route("/api/agents", axum::routing::get(handlers::list_agents))
        .route("/api/agents/:id/status", axum::routing::get(handlers::get_agent_status))
        .route("/api/agents/:id", axum::routing::delete(handlers::destroy_agent))
        .route("/api/agents/:id/tasks", axum::routing::post(handlers::send_task))
        .route("/api/agents/:id/tasks", axum::routing::get(handlers::get_agent_tasks))
        .layer(CorsLayer::permissive())
        .with_state(handlers::AppState {
            db,
            deployment_manager,
            coordinator,
        });

    Ok(router)
}
