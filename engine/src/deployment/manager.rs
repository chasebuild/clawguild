use crate::adapters::VpsAdapters;
use crate::models::{
    Agent, AgentStatus, Deployment, DeploymentStatus, VpsProvider as ModelVpsProvider,
};
use crate::runtime::RuntimeRegistry;
use crate::storage::{repositories, Database};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct DeploymentManager {
    db: Database,
    pub vps_adapters: VpsAdapters,
    runtime_registry: RuntimeRegistry,
}

impl DeploymentManager {
    pub async fn new(db: Database, vps_adapters: VpsAdapters) -> Result<Self> {
        Ok(Self {
            db,
            vps_adapters,
            runtime_registry: RuntimeRegistry::new(),
        })
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
            agent_ids: None,
            provider: provider.clone(),
            region,
            status: DeploymentStatus::Pending,
            provider_id: None,
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

        let (_runtime_kind, runtime_plan) = self
            .runtime_registry
            .build_plan(std::slice::from_ref(&agent))?;

        // Deploy to VPS with runtime configuration
        let agent_config = crate::adapters::trait_def::AgentConfig {
            agent: agent.clone(),
            agents: None,
            region: deployment.region.clone(),
            runtime: agent.runtime,
            runtime_init_script: runtime_plan.init_script,
            runtime_env: runtime_plan.env,
            runtime_services: runtime_plan.services,
        };

        let deploy_result = vps_provider.deploy_agent(agent_config).await?;

        // Persist provider_id so destroy can target the correct VPS
        deployment_repo
            .update_provider_id(deployment.id, deploy_result.provider_id.clone())
            .await?;
        deployment_repo
            .update_status(deployment.id, DeploymentStatus::Creating)
            .await?;

        // Poll deployment status until ready
        let mut attempts = 0;
        let max_attempts = 30; // 30 attempts with 2s delay = 60s timeout

        while attempts < max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let status = vps_provider.get_status(&deploy_result).await?;

            if matches!(status.status, DeploymentStatus::Running) {
                deployment_repo
                    .update_status_details(
                        deployment.id,
                        DeploymentStatus::Running,
                        status.endpoint,
                        status.gateway_url,
                    )
                    .await?;

                // Update agent status and link to deployment
                agent_repo
                    .update_status(agent.id, AgentStatus::Running)
                    .await?;
                agent_repo
                    .update_deployment_id(agent.id, Some(deployment.id))
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

    /// Deploy multiple OpenClaw agents on a single VPS. Coordination (Discord channels) is unchanged per agent.
    pub async fn deploy_agents_multi(
        &self,
        agents: Vec<Agent>,
        provider: ModelVpsProvider,
        region: Option<String>,
    ) -> Result<Deployment> {
        if agents.is_empty() {
            anyhow::bail!("At least one agent required for multi-agent deploy");
        }

        let vps_provider = self
            .vps_adapters
            .get_provider(provider.clone())
            .ok_or_else(|| anyhow::anyhow!("VPS provider {:?} not configured", provider))?;

        let agent_ids: Vec<Uuid> = agents.iter().map(|a| a.id).collect();
        let deployment = Deployment {
            id: Uuid::new_v4(),
            agent_id: agents[0].id,
            agent_ids: Some(agent_ids.clone()),
            provider: provider.clone(),
            region: region.clone(),
            status: DeploymentStatus::Pending,
            provider_id: None,
            endpoint: None,
            gateway_url: None,
            volume_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let deployment_repo = repositories::DeploymentRepository::new(self.db.db().clone());
        deployment_repo.create(&deployment).await?;

        let agent_repo = repositories::AgentRepository::new(self.db.db().clone());
        for id in &agent_ids {
            agent_repo
                .update_status(*id, AgentStatus::Deploying)
                .await?;
        }

        let (_runtime_kind, runtime_plan) = self.runtime_registry.build_plan(&agents)?;

        let agent_config = crate::adapters::trait_def::AgentConfig {
            agent: agents[0].clone(),
            agents: Some(agents.clone()),
            region: deployment.region.clone(),
            runtime: agents[0].runtime,
            runtime_init_script: runtime_plan.init_script,
            runtime_env: runtime_plan.env,
            runtime_services: runtime_plan.services,
        };

        let deploy_result = vps_provider.deploy_agent(agent_config).await?;

        deployment_repo
            .update_provider_id(deployment.id, deploy_result.provider_id.clone())
            .await?;
        deployment_repo
            .update_status(deployment.id, DeploymentStatus::Creating)
            .await?;

        let max_attempts = 30;
        let mut attempts = 0;

        while attempts < max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let status = vps_provider.get_status(&deploy_result).await?;

            if matches!(status.status, DeploymentStatus::Running) {
                deployment_repo
                    .update_status_details(
                        deployment.id,
                        DeploymentStatus::Running,
                        status.endpoint,
                        status.gateway_url,
                    )
                    .await?;

                for id in &agent_ids {
                    agent_repo.update_status(*id, AgentStatus::Running).await?;
                    agent_repo
                        .update_deployment_id(*id, Some(deployment.id))
                        .await?;
                }
                break;
            } else if matches!(status.status, DeploymentStatus::Failed) {
                deployment_repo
                    .update_status(deployment.id, DeploymentStatus::Failed)
                    .await?;
                for id in &agent_ids {
                    agent_repo.update_status(*id, AgentStatus::Error).await?;
                }
                anyhow::bail!("Multi-agent deployment failed");
            }

            attempts += 1;
        }

        if attempts >= max_attempts {
            deployment_repo
                .update_status(deployment.id, DeploymentStatus::Failed)
                .await?;
            for id in &agent_ids {
                agent_repo.update_status(*id, AgentStatus::Error).await?;
            }
            anyhow::bail!("Multi-agent deployment timeout");
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
        let _agent = agent_repo
            .get_by_id(agent_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        let deployment_repo = repositories::DeploymentRepository::new(self.db.db().clone());
        let deployment = deployment_repo.get_by_agent_id(agent_id).await?;

        if let Some(deployment) = deployment {
            // Get VPS provider adapter
            let vps_provider = self
                .vps_adapters
                .get_provider(deployment.provider.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!("VPS provider {:?} not configured", deployment.provider)
                })?;

            let deployment_id_struct = crate::adapters::trait_def::DeploymentId {
                id: deployment.id,
                provider_id: deployment
                    .provider_id
                    .unwrap_or_else(|| format!("{:?}-{}", deployment.provider, deployment.id)),
            };

            vps_provider.destroy_agent(&deployment_id_struct).await?;

            deployment_repo
                .update_status(deployment.id, DeploymentStatus::Stopped)
                .await?;

            // Mark all agents on this VPS as stopped and unlink deployment
            let agent_ids: Vec<Uuid> = deployment
                .agent_ids
                .unwrap_or_else(|| vec![deployment.agent_id]);
            for aid in agent_ids {
                agent_repo.update_status(aid, AgentStatus::Stopped).await?;
                agent_repo.update_deployment_id(aid, None).await?;
            }
        } else {
            agent_repo
                .update_status(agent_id, AgentStatus::Stopped)
                .await?;
        }

        Ok(())
    }
}
