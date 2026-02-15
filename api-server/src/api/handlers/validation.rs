use axum::extract::State;
use axum::response::Json;
use serde::Serialize;

use crate::api::errors::AppError;
use crate::api::handlers::AppState;

#[derive(Serialize)]
pub struct ServerHealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
}

pub async fn get_server_health_with_state(
    State(state): State<AppState>,
) -> Result<Json<ServerHealthResponse>, AppError> {
    Ok(Json(ServerHealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    }))
}

#[derive(Serialize)]
pub struct ServerStatusResponse {
    pub status: String,
    pub version: String,
    pub database_connected: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub async fn get_server_status(
    State(state): State<AppState>,
) -> Result<Json<ServerStatusResponse>, AppError> {
    let db_connected = sqlx::query("SELECT 1")
        .execute(&state.db.db())
        .await
        .is_ok();

    Ok(Json(ServerStatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database_connected: db_connected,
        timestamp: chrono::Utc::now(),
    }))
}
