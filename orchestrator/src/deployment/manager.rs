use crate::adapters::VpsAdapters;
use crate::models::{
    Agent, AgentStatus, Deployment, DeploymentStatus, VpsProvider as ModelVpsProvider,
};
use crate::storage::{repositories, Database};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct DeploymentManager {
    db: Database,
    vps_adapters: VpsAdapters,
}

impl DeploymentManager {
    pub async fn new(db: Database, vps_adapters: VpsAdapters) -> Result<Self> {
        Ok(Self { db, vps_adapters })
    }

    pub async fn deploy_agent(
        &self,
        agent: Agent,
        provider: ModelVpsProvider,
        region: Option<String>,
    ) -> Result<Deployment> {
        // Get the appropriate VPS provider adapter
        let vps_provider = self
            .vps_adapters
            .get_provider(provider.clone())
            .ok_or_else(|| anyhow::anyhow!("VPS provider {:?} not configured", provider))?;

        // Create deployment record
        let deployment = Deployment {
            id: Uuid::new_v4(),
            agent_id: agent.id,
            provider: provider.clone(),
            region,
            status: DeploymentStatus::Pending,
            endpoint: None,
            gateway_url: None,
            volume_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save to database
        let deployment_repo = repositories::DeploymentRepository::new(self.db.db().clone());
        deployment_repo.create(&deployment).await?;

        // Update agent status
        let agent_repo = repositories::AgentRepository::new(self.db.db().clone());
        agent_repo
            .update_status(agent.id, AgentStatus::Deploying)
            .await?;

        // Generate OpenClaw configuration and onboarding command
        let openclaw_config = crate::deployment::openclaw::OpenClawConfig::new(agent.clone());
        let config_json = openclaw_config.generate_config_json()?;
        let onboard_command = openclaw_config.generate_onboard_command()?;

        // Deploy to VPS with OpenClaw configuration
        let agent_config = crate::adapters::trait_def::AgentConfig {
            agent: agent.clone(),
            region: deployment.region.clone(),
            openclaw_onboard_command: Some(onboard_command),
            openclaw_config_json: Some(config_json),
        };

        let deployment_id = vps_provider.deploy_agent(agent_config).await?;

        // Update deployment with provider ID
        deployment_repo
            .update_status(deployment.id, DeploymentStatus::Creating)
            .await?;

        // Poll deployment status until ready
        let mut attempts = 0;
        let max_attempts = 30; // 30 attempts with 2s delay = 60s timeout

        while attempts < max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let status = vps_provider.get_status(&deployment_id).await?;

            if matches!(status.status, DeploymentStatus::Running) {
                // Update deployment with endpoint and gateway URL
                let _: Option<Deployment> = self
                    .db
                    .db()
                    .update(("deployments", deployment.id.to_string()))
                    .merge(serde_json::json!({
                        "status": DeploymentStatus::Running,
                        "endpoint": status.endpoint,
                        "gateway_url": status.gateway_url,
                        "updated_at": chrono::Utc::now(),
                    }))
                    .await?;

                // Update agent status
                agent_repo
                    .update_status(agent.id, AgentStatus::Running)
                    .await?;
                break;
            } else if matches!(status.status, DeploymentStatus::Failed) {
                deployment_repo
                    .update_status(deployment.id, DeploymentStatus::Failed)
                    .await?;
                agent_repo
                    .update_status(agent.id, AgentStatus::Error)
                    .await?;
                anyhow::bail!("Deployment failed");
            }

            attempts += 1;
        }

        if attempts >= max_attempts {
            deployment_repo
                .update_status(deployment.id, DeploymentStatus::Failed)
                .await?;
            agent_repo
                .update_status(agent.id, AgentStatus::Error)
                .await?;
            anyhow::bail!("Deployment timeout");
        }

        Ok(deployment)
    }

    pub async fn get_agent_status(&self, agent_id: Uuid) -> Result<AgentStatus> {
        let agent_repo = repositories::AgentRepository::new(self.db.db().clone());
        let agent = agent_repo
            .get_by_id(agent_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        Ok(agent.status)
    }

    pub async fn destroy_agent(&self, agent_id: Uuid) -> Result<()> {
        let agent_repo = repositories::AgentRepository::new(self.db.db().clone());
        let agent = agent_repo
            .get_by_id(agent_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        if let Some(_deployment_id) = agent.deployment_id {
            // Get deployment
            let deployment_repo = repositories::DeploymentRepository::new(self.db.db().clone());
            let deployment = deployment_repo
                .get_by_agent_id(agent_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Deployment not found"))?;

            // Get VPS provider adapter
            let vps_provider = self
                .vps_adapters
                .get_provider(deployment.provider.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!("VPS provider {:?} not configured", deployment.provider)
                })?;

            // Destroy via VPS provider
            let deployment_id_struct = crate::adapters::trait_def::DeploymentId {
                id: deployment.id,
                provider_id: format!("{:?}-{}", deployment.provider, deployment.id),
            };

            vps_provider.destroy_agent(&deployment_id_struct).await?;

            // Update deployment status
            deployment_repo
                .update_status(deployment.id, DeploymentStatus::Stopped)
                .await?;
        }

        agent_repo
            .update_status(agent_id, AgentStatus::Stopped)
            .await?;

        Ok(())
    }
}
