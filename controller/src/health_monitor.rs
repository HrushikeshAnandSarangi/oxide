use crate::metrics;
use crate::state::ControlState;
use common::events::{DeploymentEvent, EventType};
use domain::deployment::DeploymentStatus;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn start_health_monitor(state: Arc<ControlState>, shutdown: CancellationToken) {
    tracing::info!("Starting Container Health Monitor...");
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    let client = reqwest::Client::new();

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("Health monitor shutting down");
                break;
            }
            _ = interval.tick() => {}
        }

        if let Ok(active_routes) = state.deployments.get_running_deployments().await {
            for (subdomain, port) in active_routes {
                let url = format!("http://127.0.0.1:{}", port);
                let is_healthy = client
                    .get(&url)
                    .timeout(Duration::from_secs(3))
                    .send()
                    .await
                    .is_ok();

                if !is_healthy {
                    tracing::warn!(
                        "Health check failed for subdomain {} on port {}",
                        subdomain,
                        port
                    );
                    metrics::HEALTH_CHECK_FAILURES_TOTAL.inc();
                    if let Ok(Some(project)) = state.projects.find_by_subdomain(&subdomain).await
                        && let Some(deployment_id) = project.active_deployment_id
                    {
                        tracing::error!("Marking deployment {} as Crashed", deployment_id);
                        let _ = state
                            .deployments
                            .update_status(&deployment_id, DeploymentStatus::Crashed)
                            .await;
                        metrics::DEPLOYMENTS_TOTAL
                            .with_label_values(&["Crashed"])
                            .inc();
                        metrics::ACTIVE_CONTAINERS.dec();

                        if let Some(publisher) = &state.events {
                            let event = DeploymentEvent::new(
                                deployment_id,
                                &subdomain,
                                EventType::HealthCheckFailed,
                            );
                            if let Err(e) = publisher.publish(&event).await {
                                tracing::debug!("Failed to publish telemetry event: {}", e);
                            }
                        }

                        // Remove crashed route from the proxy mapper
                        state.proxy.remove_route(&subdomain);
                    }
                }
            }
        }
    }
}
