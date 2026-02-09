use anyhow::Result;
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct DiscordClient {
    token: String,
    http_client: Client,
}

#[derive(Debug, Clone)]
pub enum ChannelType {
    CoordinationLogs,
    SlaveCommunication,
    MasterOrders,
}

impl DiscordClient {
    pub async fn new(token: String) -> Result<Self> {
        Ok(Self {
            token,
            http_client: Client::new(),
        })
    }

    pub async fn start(&self) -> Result<()> {
        // Discord event handlers would be implemented here
        // For now, we use HTTP API for message sending
        // Full event handling would require a WebSocket connection via serenity
        tracing::info!("Discord client initialized and ready");
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
