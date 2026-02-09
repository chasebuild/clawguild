use anyhow::Result;
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct DiscordClient {
    #[allow(dead_code)]
    token: String,
    #[allow(dead_code)]
    http_client: Client,
}

impl DiscordClient {
    pub async fn new(token: String) -> Result<Self> {
        Ok(Self {
            token,
            http_client: Client::new(),
        })
    }

    #[allow(dead_code)]
    pub async fn start(&self) -> Result<()> {
        // Discord event handlers would be implemented here
        // For now, we use HTTP API for message sending
        // Full event handling would require a WebSocket connection via serenity
        tracing::info!("Discord client initialized");
        Ok(())
    }

    pub async fn send_message(&self, channel_id: u64, message: &str) -> Result<()> {
        let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
        
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.token))
            .header("Content-Type", "application/json")
            .json(&json!({
                "content": message
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to send Discord message: {}", error_text);
        }
        
        Ok(())
    }
}
