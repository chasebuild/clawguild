use crate::coordinator::discord::DiscordClient;
use crate::models::{Task, TaskStatus, Team};
use crate::storage::{repositories, Database};
use anyhow::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct MasterCoordinator {
    db: Database,
    discord_client: Option<DiscordClient>,
}

impl MasterCoordinator {
    pub fn new(db: Database, discord_client: Option<DiscordClient>) -> Self {
        Self { db, discord_client }
    }

    pub async fn delegate_task(&self, team: &Team, task: &Task) -> Result<Vec<Task>> {
        // Simple task breakdown: split by sentences and assign to different slaves
        let sentences: Vec<&str> = task
            .description
            .split('.')
            .filter(|s| !s.trim().is_empty())
            .collect();

        // Send master order to Discord
        if let Some(discord) = &self.discord_client {
            let order_message = format!(
                "**New Task Assigned**\nTask ID: `{}`\n{}\n\nReply with `!task-complete {}` and your result when finished.",
                task.id,
                task.description,
                task.id
            );
            discord
                .send_master_order(&team.discord_channels.master_orders, &order_message)
                .await?;
        }

        let task_repo = repositories::TaskRepository::new(self.db.db().clone());
        let mut subtasks = Vec::new();

        // If we have slave agents, delegate subtasks
        if !team.slave_ids.is_empty() && sentences.len() > 1 {
            // Distribute sentences across slaves
            for (idx, sentence) in sentences.iter().enumerate() {
                let slave_idx = idx % team.slave_ids.len();
                let slave_id = team.slave_ids[slave_idx];

                // Create subtask for slave
                let subtask = Task {
                    id: Uuid::new_v4(),
                    team_id: team.id,
                    parent_task_id: Some(task.id),
                    assigned_to: Some(slave_id),
                    status: TaskStatus::Pending,
                    description: format!("Subtask: {}", sentence.trim()),
                    result: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                task_repo.create(&subtask).await?;
                subtasks.push(subtask.clone());

                // Notify slave via Discord
                if let Some(discord) = &self.discord_client {
                    let slave_message = format!(
                        "**Subtask for Slave {}**\nTask ID: `{}`\n{}\n\nReply with `!task-complete {}` and your result when finished.",
                        slave_id, subtask.id, subtask.description, subtask.id
                    );
                    discord
                        .send_slave_message(
                            &team.discord_channels.slave_communication,
                            &slave_message,
                        )
                        .await?;
                }
            }
        }

        Ok(subtasks)
    }

    #[allow(dead_code)]
    pub async fn aggregate_results(&self, task: &Task, team: &Team) -> Result<String> {
        let task_repo = repositories::TaskRepository::new(self.db.db().clone());
        let subtasks = task_repo.get_by_parent_id(task.id).await?;

        let mut sections = Vec::new();
        if let Some(result) = &task.result {
            sections.push(format!("Main task: {}", result));
        }

        for sub in subtasks {
            if let Some(result) = sub.result {
                sections.push(format!("{}: {}", sub.description, result));
            }
        }

        let aggregated = if sections.is_empty() {
            format!("No results yet for task: {}", task.description)
        } else {
            sections.join("\n")
        };

        // Log aggregation to coordination channel
        if let Some(discord) = &self.discord_client {
            let log_message = format!(
                "**Task Completed**\nTask: {}\nResult: {}",
                task.description, aggregated
            );
            discord
                .log_coordination(&team.discord_channels.coordination_logs, &log_message)
                .await?;
        }

        Ok(aggregated)
    }
}
