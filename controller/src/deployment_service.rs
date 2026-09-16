use std::sync::Arc;
use std::time::Instant;
use anyhow::Result;
use uuid::Uuid;
use crate::state::ControlState;
use crate::metrics;
use common::events::{DeploymentEvent, EventType};

pub struct DeploymentService {
    state: Arc<ControlState>,
}

impl DeploymentService {
    pub fn new(state: Arc<ControlState>) -> Self {
        Self { state }
    }

    /// Best-effort telemetry publish — a down/unconfigured Redis must never
    /// fail or slow down a deployment, it just means this event is dropped.
    async fn emit(&self, event: DeploymentEvent) {
        if let Some(publisher) = &self.state.events {
            if let Err(e) = publisher.publish(&event).await {
                tracing::debug!("Failed to publish telemetry event: {}", e);
            }
        }
    }

    pub async fn deploy(&self, deployment_id: Uuid, subdomain: String) -> Result<()> {
        use domain::deployment::DeploymentStatus;

        self.emit(DeploymentEvent::new(deployment_id, &subdomain, EventType::DeploymentStarted)).await;

        // --- 0. Fetch Project & Env Vars ---
        let project = match self.state.projects.find_by_subdomain(&subdomain).await {
            Ok(Some(p)) => p,
            _ => return Err(anyhow::anyhow!("Project not found for subdomain {}", subdomain)),
        };

        let repo_url = project.repo_url.clone().ok_or_else(|| anyhow::anyhow!("No repository URL configured for project {}", subdomain))?;

        let mut env_strings = Vec::new();
        if let Ok(env_rows) = self.state.projects.get_env_vars(project.id).await {
            let key_str = std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| "00000000000000000000000000000000".to_string());
            let mut key_bytes = [0u8; 32];
            let bytes = key_str.as_bytes();
            let len = bytes.len().min(32);
            key_bytes[..len].copy_from_slice(&bytes[..len]);

            for row in env_rows {
                if let Ok(plaintext) = common::crypto::decrypt(&row.value_encrypted, &row.nonce, &key_bytes) {
                    env_strings.push(format!("{}={}", row.key, plaintext));
                }
            }
        }
        let env_vars = if env_strings.is_empty() { None } else { Some(env_strings) };
        
        // --- 1. Queue to Building ---
        tracing::info!("Starting Deployment for {} with version {}", deployment_id, "v1");
        
        let mut new_deployment = domain::deployment::Deployment::new(project.id, "v1".to_string());
        new_deployment.id = deployment_id;
        
        if let Err(e) = self.state.deployments.create(&new_deployment).await {
            tracing::warn!("Failed to create deployment record: {}", e);
        }

        if let Err(e) = self.state.deployments.update_status(&deployment_id, DeploymentStatus::Building).await {
            tracing::warn!("Failed to update deployment status to Building: {}", e);
        }

        // --- 2. Build Execution ---
        self.emit(DeploymentEvent::new(deployment_id, &subdomain, EventType::BuildStarted)).await;
        let build_start = Instant::now();
        let artifact = match self.state.builder.build(&repo_url, deployment_id).await {
            Ok(artifact) => artifact,
            Err(e) => {
                let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::BuildFailed).await;
                metrics::DEPLOYMENTS_TOTAL.with_label_values(&["BuildFailed"]).inc();
                self.emit(
                    DeploymentEvent::new(deployment_id, &subdomain, EventType::BuildFailed)
                        .with_duration_ms(build_start.elapsed().as_millis() as i64),
                )
                .await;
                return Err(e.into());
            }
        };
        let build_duration = build_start.elapsed();
        metrics::BUILD_DURATION_SECONDS.observe(build_duration.as_secs_f64());
        self.emit(
            DeploymentEvent::new(deployment_id, &subdomain, EventType::BuildCompleted)
                .with_duration_ms(build_duration.as_millis() as i64),
        )
        .await;

        // --- 3. Image Building & Container Starting ---
        tracing::info!("Build Completed. Building Image...");
        let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::ImageBuilding).await;

        let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::ContainerStarting).await;
        let (container_id, port) = match self.state.runtime.deploy(artifact, deployment_id, env_vars).await {
            Ok(res) => res,
            Err(e) => {
                let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::Crashed).await;
                metrics::DEPLOYMENTS_TOTAL.with_label_values(&["Crashed"]).inc();
                self.emit(DeploymentEvent::new(deployment_id, &subdomain, EventType::DeploymentCrashed)).await;
                return Err(e.into());
            }
        };
        metrics::ACTIVE_CONTAINERS.inc();
        self.emit(DeploymentEvent::new(deployment_id, &subdomain, EventType::ContainerStarted)).await;

        // --- 4. Running & Route Setup ---
        tracing::info!("{} container started", container_id);
        self.state.proxy.add_route(subdomain.clone(), port);

        if let Err(e) = self.state.deployments.set_container_info(&deployment_id, &container_id, port).await {
            tracing::warn!("Failed to persist container info: {}", e);
        }
        let _ = self.state.deployments.update_status(&deployment_id, DeploymentStatus::Running).await;
        // Also update the active deployment id for the project
        let _ = self.state.projects.update_active_deployment(&project.id, &deployment_id).await;
        tracing::info!("Route activated {} -> {}", subdomain, port);
        metrics::DEPLOYMENTS_TOTAL.with_label_values(&["Running"]).inc();
        self.emit(DeploymentEvent::new(deployment_id, &subdomain, EventType::RouteActivated)).await;
        self.emit(DeploymentEvent::new(deployment_id, &subdomain, EventType::DeploymentCompleted)).await;

        // --- 5. Clean up old container ---
        if let Some(old_deployment_id) = project.active_deployment_id {
            if let Ok(Some(old_deployment)) = self.state.deployments.find_by_id(&old_deployment_id).await {
                if let Some(old_container_id) = old_deployment.container_id {
                    tracing::info!("Stopping old container {}...", old_container_id);
                    let _ = self.state.runtime.stop(&old_container_id).await;
                    let _ = self.state.runtime.remove(&old_container_id).await;
                    let _ = self.state.deployments.update_status(&old_deployment_id, DeploymentStatus::Stopped).await;
                    metrics::ACTIVE_CONTAINERS.dec();
                }
            }
        }

        Ok(())
    }
}
