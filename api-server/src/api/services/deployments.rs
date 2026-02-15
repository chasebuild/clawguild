use crate::api::errors::AppError;
use crate::api::handlers::AppState;
use engine::models::Deployment;
use engine::storage::repositories::DeploymentRepository;
use uuid::Uuid;

pub struct DeploymentService<'a> {
    state: &'a AppState,
}

impl<'a> DeploymentService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list_deployments(&self) -> Result<Vec<Deployment>, AppError> {
        let repo = DeploymentRepository::new(self.state.db.db().clone());
        let deployments = repo
            .list_all()
            .await
            .map_err(|err| AppError::Internal(err.into()))?;
        Ok(deployments)
    }

    pub async fn get_deployment(&self, id: Uuid) -> Result<Deployment, AppError> {
        let repo = DeploymentRepository::new(self.state.db.db().clone());
        repo.get_by_id(id)
            .await
            .map_err(|err| AppError::Internal(err.into()))?
            .ok_or_else(|| AppError::NotFound("deployment not found".to_string()))
    }

    pub async fn get_deployment_logs(
        &self,
        id: Uuid,
        lines: Option<i32>,
    ) -> Result<Vec<String>, AppError> {
        let repo = DeploymentRepository::new(self.state.db.db().clone());
        let deployment = repo
            .get_by_id(id)
            .await
            .map_err(|err| AppError::Internal(err.into()))?
            .ok_or_else(|| AppError::NotFound("deployment not found".to_string()))?;

        let provider = self
            .state
            .deployment_manager
            .vps_adapters
            .get_provider(deployment.provider.clone())
            .ok_or_else(|| AppError::BadRequest("vps provider not configured".to_string()))?;

        let deployment_id = engine::adapters::trait_def::DeploymentId {
            id: deployment.id,
            provider_id: deployment
                .provider_id
                .unwrap_or_else(|| format!("{:?}-{}", deployment.provider, deployment.id)),
        };

        let lines = lines.and_then(|value| usize::try_from(value).ok());
        provider
            .get_logs(&deployment_id, lines)
            .await
            .map_err(|err| AppError::Internal(err.into()))
    }
}
