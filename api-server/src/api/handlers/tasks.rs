use axum::extract::{Path, State};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::errors::AppError;
use crate::api::handlers::AppState;
use crate::api::services::tasks::TaskService;
use engine::models::{Task, TaskStatus};

#[derive(Deserialize)]
pub struct SendTaskRequest {
    pub description: String,
}

#[derive(Serialize)]
pub struct TaskAggregateResponse {
    pub tasks: Vec<Task>,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<TaskStatus>,
    pub result: Option<String>,
}

pub async fn send_task(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<SendTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let service = TaskService::new(&state);
    let task = service.send_task(agent_id, req.description).await?;
    Ok(Json(task))
}

pub async fn get_agent_tasks(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<Task>>, AppError> {
    let service = TaskService::new(&state);
    let tasks = service.get_agent_tasks(agent_id).await?;
    Ok(Json(tasks))
}

pub async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let service = TaskService::new(&state);
    let task = service.update_task(task_id, req).await?;
    Ok(Json(task))
}

pub async fn aggregate_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskAggregateResponse>, AppError> {
    let service = TaskService::new(&state);
    let response = service.aggregate_task(task_id).await?;
    Ok(Json(response))
}
