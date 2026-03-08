use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use crate::state::ControlState;

pub struct DeploymentService {
    state: Arc<ControlState>,
}

impl DeploymentService {
    pub fn new(state: Arc<ControlState>) -> Self {
        Self { state }
    }

    pub async fn deploy(&self, subdomain: String, repo_url: String) -> Result<()> {
        use domain::deployment::DeploymentStatus;
        
        let deployment_id = Uuid::new_v4();
        
        // --- 1. Queue to Building ---
        tracing::info!("Starting Deployment for {}", deployment_id);
        if let Err(e) = self.state.deployments.update_status(&deployment_id, DeploymentStatus::Building).await {
            tracing::warn!("Failed to update deployment status to Building: {}", e);
        }

        // --- 2. Build Execution ---
        let artifact = match self.state.builder.build(&repo_url, deployment_id).await {
            Ok(artifact) => artifact,
            Err(e) => {
                let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::BuildFailed).await;
                return Err(e.into());
            }
        };

        // --- 3. Image Building & Container Starting ---
        tracing::info!("Build Completed. Building Image...");
        let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::ImageBuilding).await;
        
        let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::ContainerStarting).await;
        let (container_id, port) = match self.state.runtime.deploy(artifact, deployment_id).await {
            Ok(res) => res,
            Err(e) => {
                let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::Crashed).await;
                return Err(e.into());
            }
        };

        // --- 4. Running & Route Setup ---
        tracing::info!("{} container started", container_id);
        self.state.proxy.add_route(subdomain.clone(), port);
        
        let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::Running).await;
        tracing::info!("Route activated {} -> {}", subdomain, port);
        
        Ok(())
    }
}
