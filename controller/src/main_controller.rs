use std::sync::Arc;
use anyhow::Result;
use crate::state::ControlState;
use crate::deployment_service::DeploymentService;

pub struct ControlPlane {
    state: Arc<ControlState>,
}

impl ControlPlane {
    pub fn new(state: ControlState) -> Self {
        Self {
            state: Arc::new(state),
        }
    }

    pub async fn deploy(&self, project_subdomain: String, repo_url: String) -> Result<()> {
        let service = DeploymentService::new(self.state.clone());
        service.deploy(project_subdomain, repo_url).await
    }
}
