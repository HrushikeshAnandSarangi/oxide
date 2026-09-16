use std::sync::Arc;

use builder::Builder;
use common::events::EventPublisher;
use db::deployment_repo::DeploymentRepository;
use db::project_repo::ProjectRepository;
use proxy::state::ProxyState;
use runtime::runtime::Runtime;

#[derive(Clone)]
pub struct ControlState {
    pub builder: Arc<Builder>,
    pub runtime: Arc<Runtime>,
    pub proxy: Arc<ProxyState>,
    pub deployments: Arc<DeploymentRepository>,
    pub projects: Arc<ProjectRepository>,
    /// None when Redis isn't configured/reachable — telemetry is best-effort
    /// and must never block or fail a deployment.
    pub events: Option<Arc<EventPublisher>>,
}
