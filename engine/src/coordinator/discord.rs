use anyhow::Result;
use crate::models::TaskStatus;
use crate::storage::{repositories, Database};
use reqwest::Client;
use serde_json::json;
use serenity::{
    client::Client as SerenityClient,
    model::channel::Message,
    prelude::{Context, EventHandler, GatewayIntents},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct DiscordClient {
    token: String,
    http_client: Client,
    db: Database,
}

#[derive(Debug, Clone)]
pub enum ChannelType {
    CoordinationLogs,
    SlaveCommunication,
    MasterOrders,
}

impl DiscordClient {
    pub async fn new(token: String, db: Database) -> Result<Self> {
        Ok(Self {
            token,
            http_client: Client::new(),
            db,
        })
    }

    pub async fn start(&self) -> Result<()> {
        // Start gateway for inbound message handling while keeping HTTP for outbound messages.
        let intents =
            GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let handler = DiscordEventHandler {
            db: self.db.clone(),
        };

        let mut client = SerenityClient::builder(&self.token, intents)
            .event_handler(handler)
            .await?;

        tokio::spawn(async move {
            if let Err(error) = client.start().await {
                tracing::error!("Discord gateway error: {}", error);
            }
        });

        tracing::info!("Discord client initialized with gateway and HTTP API");
        Ok(())
    }

    pub async fn send_message(&self, channel_id: u64, message: &str) -> Result<()> {
        self.send_message_with_embed(channel_id, message, None)
            .await
    }

    pub async fn send_message_with_embed(
        &self,
        channel_id: u64,
        message: &str,
        embed: Option<serde_json::Value>,
    ) -> Result<()> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            channel_id
        );

        let mut payload = json!({
            "content": message
        });

        if let Some(embed_data) = embed {
            payload["embeds"] = json!([embed_data]);
        }

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to send Discord message: {}", error_text);
        }

        Ok(())
    }

    pub async fn send_to_channel_type(
        &self,
        channel_id: &str,
        channel_type: ChannelType,
        message: &str,
    ) -> Result<()> {
        let channel_id_u64: u64 = channel_id
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid channel ID: {}", channel_id))?;

        let prefix = match channel_type {
            ChannelType::CoordinationLogs => "📊 [COORD]",
            ChannelType::SlaveCommunication => "🤖 [SLAVE]",
            ChannelType::MasterOrders => "👑 [MASTER]",
        };

        let formatted_message = format!("{} {}", prefix, message);
        self.send_message(channel_id_u64, &formatted_message).await
    }

    pub async fn log_coordination(&self, channel_id: &str, message: &str) -> Result<()> {
        self.send_to_channel_type(channel_id, ChannelType::CoordinationLogs, message)
            .await
    }

    pub async fn send_slave_message(&self, channel_id: &str, message: &str) -> Result<()> {
        self.send_to_channel_type(channel_id, ChannelType::SlaveCommunication, message)
            .await
    }

    pub async fn send_master_order(&self, channel_id: &str, message: &str) -> Result<()> {
        self.send_to_channel_type(channel_id, ChannelType::MasterOrders, message)
            .await
    }
}

struct DiscordEventHandler {
    db: Database,
}

#[serenity::async_trait]
impl EventHandler for DiscordEventHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if let Some((task_id, result)) = parse_task_complete(&msg.content) {
            let repo = repositories::TaskRepository::new(self.db.db().clone());
            let update = repo
                .update_fields(task_id, Some(TaskStatus::Completed), Some(result.clone()))
                .await;

            match update {
                Ok(_) => {
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, format!("Task {} marked completed.", task_id))
                        .await;
                }
                Err(error) => {
                    tracing::error!("Failed updating task {}: {}", task_id, error);
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, "Failed to update task status.")
                        .await;
                }
            }
        }
    }
}

fn parse_task_complete(content: &str) -> Option<(Uuid, String)> {
    let trimmed = content.trim();
    if !trimmed.starts_with("!task-complete") {
        return None;
    }

    let mut parts = trimmed.splitn(3, ' ');
    let _ = parts.next();
    let task_id = parts.next()?;
    let result = parts.next().unwrap_or("").trim();

    let task_id = Uuid::parse_str(task_id).ok()?;
    let result = if result.is_empty() {
        "Completed via Discord".to_string()
    } else {
        result.to_string()
    };

    Some((task_id, result))
}
