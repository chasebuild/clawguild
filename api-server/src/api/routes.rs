use axum::Router;
use tower_http::cors::CorsLayer;
use engine::deployment::manager::DeploymentManager;
use engine::coordinator::Coordinator;
use engine::storage::Database;
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
        .route("/api/teams/:id/roster", axum::routing::get(handlers::get_team_roster))
        .layer(CorsLayer::permissive())
        .with_state(handlers::AppState {
            db,
            deployment_manager,
            coordinator,
        });

    Ok(router)
}
