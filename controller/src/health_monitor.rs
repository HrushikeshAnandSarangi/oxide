use std::sync::Arc;
use std::time::Duration;
use crate::state::ControlState;
use domain::deployment::DeploymentStatus;

pub async fn start_health_monitor(state: Arc<ControlState>) {
    tracing::info!("Starting Container Health Monitor...");
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    let client = reqwest::Client::new();

    loop {
        interval.tick().await;

        if let Ok(active_routes) = state.deployments.get_running_deployments().await {
            for (subdomain, port) in active_routes {
                let url = format!("http://127.0.0.1:{}", port);
                let is_healthy = client.get(&url).timeout(Duration::from_secs(3)).send().await.is_ok();

                if !is_healthy {
                    tracing::warn!("Health check failed for subdomain {} on port {}", subdomain, port);
                    if let Ok(Some(project)) = state.projects.find_by_subdomain(&subdomain).await {
                        if let Some(deployment_id) = project.active_deployment_id {
                            tracing::error!("Marking deployment {} as Crashed", deployment_id);
                            let _ = state.deployments.update_status(&deployment_id, DeploymentStatus::Crashed).await;
                            
                            // Remove crashed route from the proxy mapper
                            state.proxy.remove_route(&subdomain);
                        }
                    }
                }
            }
        }
    }
}
