use crate::api::handlers;
use axum::Router;
use engine::coordinator::Coordinator;
use engine::deployment::manager::DeploymentManager;
use engine::storage::Database;
use tower_http::cors::CorsLayer;

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
        .route(
            "/api/agents/deploy-multi",
            axum::routing::post(handlers::deploy_agents_multi),
        )
        .route(
            "/api/agents/:id/status",
            axum::routing::get(handlers::get_agent_status),
        )
        .route(
            "/api/agents/:id",
            axum::routing::delete(handlers::destroy_agent),
        )
        .route(
            "/api/agents/:id/tasks",
            axum::routing::post(handlers::send_task),
        )
        .route(
            "/api/agents/:id/tasks",
            axum::routing::get(handlers::get_agent_tasks),
        )
        .route(
            "/api/teams/:id/roster",
            axum::routing::get(handlers::get_team_roster),
        )
        .route(
            "/api/server/health",
            axum::routing::get(handlers::get_server_health),
        )
        .route(
            "/api/server/status",
            axum::routing::get(handlers::get_server_status),
        )
        .route(
            "/api/deployments",
            axum::routing::get(handlers::list_deployments),
        )
        .route(
            "/api/deployments/:id",
            axum::routing::get(handlers::get_deployment),
        )
        .route(
            "/api/deployments/:id/logs",
            axum::routing::get(handlers::get_deployment_logs),
        )
        .layer(CorsLayer::permissive())
        .with_state(handlers::AppState {
            db,
            deployment_manager,
            coordinator,
        });

    Ok(router)
}
