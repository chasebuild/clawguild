use axum::extract::{Path, Query, State};
use axum::response::Json;
use engine::models::Deployment;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::errors::AppError;
use crate::api::handlers::AppState;
use crate::api::services::deployments::DeploymentService;

#[derive(Serialize)]
pub struct DeploymentResponse {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub agent_ids: Option<Vec<Uuid>>,
    pub provider: String,
    pub region: Option<String>,
    pub status: String,
    pub endpoint: Option<String>,
    pub gateway_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Deployment> for DeploymentResponse {
    fn from(deployment: Deployment) -> Self {
        Self {
            id: deployment.id,
            agent_id: deployment.agent_id,
            agent_ids: deployment.agent_ids,
            provider: format!("{:?}", deployment.provider),
            region: deployment.region,
            status: format!("{:?}", deployment.status),
            endpoint: deployment.endpoint,
            gateway_url: deployment.gateway_url,
            created_at: deployment.created_at,
            updated_at: deployment.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct DeploymentLogsQuery {
    pub lines: Option<i32>,
}

pub async fn list_deployments(
    State(state): State<AppState>,
) -> Result<Json<Vec<DeploymentResponse>>, AppError> {
    let service = DeploymentService::new(&state);
    let deployments = service.list_deployments().await?;
    Ok(Json(
        deployments
            .into_iter()
            .map(DeploymentResponse::from)
            .collect(),
    ))
}

pub async fn get_deployment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeploymentResponse>, AppError> {
    let service = DeploymentService::new(&state);
    let deployment = service.get_deployment(id).await?;
    Ok(Json(deployment.into()))
}

pub async fn get_deployment_logs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<DeploymentLogsQuery>,
) -> Result<Json<Vec<String>>, AppError> {
    let service = DeploymentService::new(&state);
    let logs = service.get_deployment_logs(id, query.lines).await?;
    Ok(Json(logs))
}
