pub mod discord;
pub mod master;
pub mod slave;

use crate::storage::Database;
use anyhow::Result;

#[derive(Clone)]
pub struct Coordinator {
    db: Database,
    discord_client: Option<discord::DiscordClient>,
    master_coordinator: master::MasterCoordinator,
    slave_coordinator: slave::SlaveCoordinator,
}

impl Coordinator {
    pub async fn new(db: Database, discord_bot_token: Option<String>) -> Result<Self> {
        let discord_client = if let Some(token) = discord_bot_token {
            let client = discord::DiscordClient::new(token, db.clone()).await?;
            // Initialize the Discord client (start event handlers if needed)
            client.start().await?;
            Some(client)
        } else {
            None
        };

        let master_coordinator = master::MasterCoordinator::new(db.clone(), discord_client.clone());
        let slave_coordinator = slave::SlaveCoordinator::new(db.clone(), discord_client.clone());

        Ok(Self {
            db,
            discord_client,
            master_coordinator,
            slave_coordinator,
        })
    }

    pub fn master(&self) -> &master::MasterCoordinator {
        &self.master_coordinator
    }

    pub fn slave(&self) -> &slave::SlaveCoordinator {
        &self.slave_coordinator
    }

    pub async fn log_coordination(&self, channel_id: &str, message: &str) -> Result<()> {
        if let Some(discord) = &self.discord_client {
            discord.log_coordination(channel_id, message).await?;
        }
        Ok(())
    }
}
