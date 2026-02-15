pub mod agents;
pub mod channels;
pub mod deployments;
pub mod tasks;
pub mod teams;
pub mod validation;

use engine::coordinator::Coordinator;
use engine::deployment::manager::DeploymentManager;
use engine::storage::Database;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub deployment_manager: DeploymentManager,
    #[allow(dead_code)]
    pub coordinator: Coordinator,
    pub api_key: Option<String>,
    pub start_time: Instant,
}

pub use agents::{create_agent, deploy_agents_multi, destroy_agent, get_agent_status, list_agents};
pub use deployments::{get_deployment, get_deployment_logs, list_deployments};
pub use tasks::{aggregate_task, get_agent_tasks, send_task, update_task};
pub use teams::{assign_agent_to_team, create_team, get_team_roster, list_teams};
pub use validation::{get_server_health_with_state, get_server_status};
