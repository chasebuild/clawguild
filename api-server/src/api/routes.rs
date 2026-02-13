use crate::api::handlers;
use crate::api::middleware;
use axum::Router;
use engine::coordinator::Coordinator;
use engine::deployment::manager::DeploymentManager;
use engine::storage::Database;
use tower_http::cors::CorsLayer;
use axum::middleware as axum_middleware;

pub async fn create_router(
    db: Database,
    deployment_manager: DeploymentManager,
    coordinator: Coordinator,
    api_key: Option<String>,
    start_time: std::time::Instant,
) -> anyhow::Result<Router> {
    let state = handlers::AppState {
        db,
        deployment_manager,
        coordinator,
        api_key,
        start_time,
    };

    let router = Router::new()
        .route("/api/teams", axum::routing::post(handlers::create_team))
        .route("/api/teams", axum::routing::get(handlers::list_teams))
        .route(
            "/api/teams/:id/assign",
            axum::routing::post(handlers::assign_agent_to_team),
        )
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
            axum::routing::get(handlers::get_server_health_with_state),
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
        .route(
            "/api/tasks/:id",
            axum::routing::patch(handlers::update_task),
        )
        .route(
            "/api/tasks/:id/aggregate",
            axum::routing::get(handlers::aggregate_task),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::require_api_key,
        ))
        .layer(CorsLayer::permissive())
        .with_state(state);

    Ok(router)
}
