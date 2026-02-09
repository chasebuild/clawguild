pub mod master;
pub mod slave;
pub mod discord;

use anyhow::Result;
use crate::storage::Database;

#[derive(Clone)]
pub struct Coordinator {
    #[allow(dead_code)]
    db: Database,
    #[allow(dead_code)]
    discord_client: Option<discord::DiscordClient>,
}

impl Coordinator {
    pub async fn new(db: Database, discord_bot_token: Option<String>) -> Result<Self> {
        let discord_client = if let Some(token) = discord_bot_token {
            Some(discord::DiscordClient::new(token).await?)
        } else {
            None
        };

        Ok(Self {
            db,
            discord_client,
        })
    }
}
