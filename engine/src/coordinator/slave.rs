use crate::coordinator::discord::DiscordClient;
use crate::models::{Task, TaskStatus, Team};
use crate::storage::{repositories, Database};
use anyhow::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct SlaveCoordinator {
    db: Database,
    discord_client: Option<DiscordClient>,
}

impl SlaveCoordinator {
    pub fn new(db: Database, discord_client: Option<DiscordClient>) -> Self {
        Self { db, discord_client }
    }

    pub async fn execute_task(&self, _task: &Task) -> Result<String> {
        // Task execution is handled by the OpenClaw agent instance
        // This coordinator just tracks the task status
        // The actual execution happens when the OpenClaw agent receives the task via Discord

        // Simulate task execution result
        Ok("Task executed by OpenClaw agent".to_string())
    }

    pub async fn report_result(&self, task_id: Uuid, result: &str, team: &Team) -> Result<()> {
        let task_repo = repositories::TaskRepository::new(self.db.db().clone());
        task_repo
            .update_fields(
                task_id,
                Some(TaskStatus::Completed),
                Some(result.to_string()),
            )
            .await?;

        // Send result to slave communication channel
        if let Some(discord) = &self.discord_client {
            let message = format!("**Task {} Completed**\nResult: {}", task_id, result);
            discord
                .send_slave_message(&team.discord_channels.slave_communication, &message)
                .await?;

            // Also log to coordination channel
            let log_message = format!("Slave reported completion for task {}: {}", task_id, result);
            discord
                .log_coordination(&team.discord_channels.coordination_logs, &log_message)
                .await?;
        }

        tracing::info!("Task {} result reported: {}", task_id, result);
        Ok(())
    }
}
