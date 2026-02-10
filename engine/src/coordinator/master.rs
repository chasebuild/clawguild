use crate::coordinator::discord::DiscordClient;
use crate::models::{Task, TaskStatus, Team};
use anyhow::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct MasterCoordinator {
    discord_client: Option<DiscordClient>,
}

impl MasterCoordinator {
    pub fn new(discord_client: Option<DiscordClient>) -> Self {
        Self { discord_client }
    }

    pub async fn delegate_task(&self, team: &Team, task_description: &str) -> Result<Task> {
        // Simple task breakdown: split by sentences and assign to different slaves
        let sentences: Vec<&str> = task_description
            .split('.')
            .filter(|s| !s.trim().is_empty())
            .collect();

        // Create main task
        let main_task = Task {
            id: Uuid::new_v4(),
            team_id: team.id,
            assigned_to: Some(team.master_id),
            status: TaskStatus::Pending,
            description: task_description.to_string(),
            result: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Send master order to Discord
        if let Some(discord) = &self.discord_client {
            let order_message = format!("**New Task Assigned**\n{}", task_description);
            discord
                .send_master_order(&team.discord_channels.master_orders, &order_message)
                .await?;
        }

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
                    assigned_to: Some(slave_id),
                    status: TaskStatus::Pending,
                    description: format!("Subtask: {}", sentence.trim()),
                    result: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                // Notify slave via Discord
                if let Some(discord) = &self.discord_client {
                    let slave_message = format!(
                        "**Subtask for Slave {}**\n{}",
                        slave_id, subtask.description
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

        Ok(main_task)
    }

    #[allow(dead_code)]
    pub async fn aggregate_results(&self, task: &Task, team: &Team) -> Result<String> {
        // This would query all related tasks and combine their results
        let aggregated = format!("Results aggregated for task: {}", task.description);

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
