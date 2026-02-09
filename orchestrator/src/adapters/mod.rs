pub mod trait_def;
pub mod railway;
pub mod flyio;
pub mod aws;

use anyhow::Result;
use crate::config::Config;
use std::sync::Arc;

pub use trait_def::VpsProvider;

#[derive(Clone)]
pub struct VpsAdapters {
    pub railway: Option<Arc<dyn VpsProvider>>,
    pub flyio: Option<Arc<dyn VpsProvider>>,
    pub aws: Option<Arc<dyn VpsProvider>>,
}

impl VpsAdapters {
    pub async fn new(config: &Config) -> Result<Self> {
        let railway = if config.railway_api_key.is_some() {
            Some(Arc::new(railway::RailwayAdapter::new(config)?) as Arc<dyn VpsProvider>)
        } else {
            None
        };

        let flyio = if config.fly_api_token.is_some() {
            Some(Arc::new(flyio::FlyIoAdapter::new(config)?) as Arc<dyn VpsProvider>)
        } else {
            None
        };

        let aws = if config.aws_access_key_id.is_some() && config.aws_secret_access_key.is_some() {
            Some(Arc::new(aws::AwsAdapter::new(config)?) as Arc<dyn VpsProvider>)
        } else {
            None
        };

        Ok(VpsAdapters {
            railway,
            flyio,
            aws,
        })
    }

    pub fn get_provider(&self, provider: crate::models::VpsProvider) -> Option<Arc<dyn VpsProvider>> {
        match provider {
            crate::models::VpsProvider::Railway => self.railway.clone(),
            crate::models::VpsProvider::FlyIo => self.flyio.clone(),
            crate::models::VpsProvider::Aws => self.aws.clone(),
        }
    }
}
