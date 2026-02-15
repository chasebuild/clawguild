use crate::api::errors::AppError;
use crate::api::handlers::tasks::{TaskAggregateResponse, UpdateTaskRequest};
use crate::api::handlers::AppState;
use engine::models::{AgentRole, Task, TaskStatus};
use engine::storage::repositories::{AgentRepository, TaskRepository, TeamRepository};
use uuid::Uuid;

pub struct TaskService<'a> {
    state: &'a AppState,
}

impl<'a> TaskService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn send_task(&self, agent_id: Uuid, description: String) -> Result<Task, AppError> {
        let agent_repo = AgentRepository::new(self.state.db.db().clone());
        let agent = agent_repo
            .get_by_id(agent_id)
            .await
            .map_err(|err| AppError::Internal(err.into()))?
            .ok_or_else(|| AppError::NotFound("agent not found".to_string()))?;

        let team_id = agent
            .team_id
            .ok_or_else(|| AppError::BadRequest("agent not in a team".to_string()))?;

        let mut task = Task {
            id: Uuid::new_v4(),
            team_id,
            parent_task_id: None,
            assigned_to: Some(agent_id),
            status: TaskStatus::Pending,
            description,
            result: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let task_repo = TaskRepository::new(self.state.db.db().clone());
        task_repo
            .create(&task)
            .await
            .map_err(|err| AppError::Internal(err.into()))?;

        if matches!(agent.role, AgentRole::Master) {
            let team_repo = TeamRepository::new(self.state.db.db().clone());
            let team = team_repo
                .get_by_id(team_id)
                .await
                .map_err(|err| AppError::Internal(err.into()))?
                .ok_or_else(|| AppError::NotFound("team not found".to_string()))?;

            self.state
                .coordinator
                .master()
                .delegate_task(&team, &task)
                .await
                .map_err(|err| AppError::Internal(err.into()))?;

            task_repo
                .update_fields(task.id, Some(TaskStatus::InProgress), None)
                .await
                .map_err(|err| AppError::Internal(err.into()))?;
            task.status = TaskStatus::InProgress;
        }

        Ok(task)
    }

    pub async fn get_agent_tasks(&self, agent_id: Uuid) -> Result<Vec<Task>, AppError> {
        let task_repo = TaskRepository::new(self.state.db.db().clone());
        let tasks = task_repo
            .get_by_agent_id(agent_id)
            .await
            .map_err(|err| AppError::Internal(err.into()))?;
        Ok(tasks)
    }

    pub async fn update_task(
        &self,
        task_id: Uuid,
        req: UpdateTaskRequest,
    ) -> Result<Task, AppError> {
        let task_repo = TaskRepository::new(self.state.db.db().clone());
        let updated = task_repo
            .update_fields(task_id, req.status, req.result)
            .await
            .map_err(|err| AppError::Internal(err.into()))?
            .ok_or_else(|| AppError::NotFound("task not found".to_string()))?;
        Ok(updated)
    }

    pub async fn aggregate_task(&self, task_id: Uuid) -> Result<TaskAggregateResponse, AppError> {
        let task_repo = TaskRepository::new(self.state.db.db().clone());
        let task = task_repo
            .get_by_id(task_id)
            .await
            .map_err(|err| AppError::Internal(err.into()))?
            .ok_or_else(|| AppError::NotFound("task not found".to_string()))?;

        let subtasks = task_repo
            .get_by_parent_id(task_id)
            .await
            .map_err(|err| AppError::Internal(err.into()))?;

        let mut results = vec![task];
        results.extend(subtasks);

        Ok(TaskAggregateResponse { tasks: results })
    }
}
