use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub discord_bot_token: Option<String>,
    pub railway_api_key: Option<String>,
    pub fly_api_token: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub openclaw_api_key: Option<String>,
    pub api_key: Option<String>,
    pub api_port: u16,
    pub api_host: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Config {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/clawguild".to_string()
            }),
            discord_bot_token: env::var("DISCORD_BOT_TOKEN").ok(),
            railway_api_key: env::var("RAILWAY_API_KEY").ok(),
            fly_api_token: env::var("FLY_API_TOKEN").ok(),
            aws_access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
            aws_secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            openclaw_api_key: env::var("OPENCLAW_API_KEY").ok(),
            api_key: env::var("API_KEY").ok(),
            api_port: env::var("API_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            api_host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        })
    }
}
