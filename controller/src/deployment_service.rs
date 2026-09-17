use crate::metrics;
use crate::state::ControlState;
use anyhow::Result;
use common::events::{DeploymentEvent, EventType};
use db::models::ProjectRow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

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
        if let Some(publisher) = &self.state.events
            && let Err(e) = publisher.publish(&event).await
        {
            tracing::debug!("Failed to publish telemetry event: {}", e);
        }
    }

    /// Sets up the log channel for a deployment and spawns the task that
    /// drains it into `deployments.build_log` line-by-line, so the
    /// dashboard can show build/image-build progress live instead of only
    /// the final outcome. Returns the sender half to pass into the
    /// builder/runtime, plus the drain task's handle so the caller can wait
    /// for the last lines to actually land before returning.
    fn spawn_log_drain(
        &self,
        deployment_id: Uuid,
    ) -> (common::buildlog::LogSender, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let deployments = self.state.deployments.clone();
        let handle = tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                if let Err(e) = deployments.append_build_log(&deployment_id, &line).await {
                    tracing::debug!("Failed to append build log line: {}", e);
                }
            }
        });
        (tx, handle)
    }

    pub async fn deploy(&self, deployment_id: Uuid, subdomain: String) -> Result<()> {
        use domain::deployment::DeploymentStatus;

        self.emit(DeploymentEvent::new(
            deployment_id,
            &subdomain,
            EventType::DeploymentStarted,
        ))
        .await;

        // --- 0. Fetch Project & Env Vars ---
        let project = match self.state.projects.find_by_subdomain(&subdomain).await {
            Ok(Some(p)) => p,
            _ => {
                return Err(anyhow::anyhow!(
                    "Project not found for subdomain {}",
                    subdomain
                ));
            }
        };

        let repo_url = project.repo_url.clone().ok_or_else(|| {
            anyhow::anyhow!("No repository URL configured for project {}", subdomain)
        })?;

        let env_vars = self.decrypt_env_vars(&project).await;

        // --- 1. Queue to Building ---
        tracing::info!(
            "Starting Deployment for {} with version {}",
            deployment_id,
            "v1"
        );

        // The row already exists — ControlPlane::deploy() creates it (status
        // Queued) before handing off to this worker, so the caller gets a
        // valid deployment_id back immediately rather than waiting on the
        // queue to drain.
        if let Err(e) = self
            .state
            .deployments
            .update_status(&deployment_id, DeploymentStatus::Building)
            .await
        {
            tracing::warn!("Failed to update deployment status to Building: {}", e);
        }

        // --- 2. Build Execution ---
        self.emit(DeploymentEvent::new(
            deployment_id,
            &subdomain,
            EventType::BuildStarted,
        ))
        .await;
        let (log_tx, log_drain) = self.spawn_log_drain(deployment_id);
        let build_start = Instant::now();
        let artifact = match self
            .state
            .builder
            .build(
                &repo_url,
                deployment_id,
                project.auto_generate_flake,
                Some(log_tx.clone()),
            )
            .await
        {
            Ok(artifact) => artifact,
            Err(e) => {
                // Don't keep a row for a deployment that never worked — the
                // failure is still fully recorded in the BuildFailed metric
                // and the telemetry event just below, both emitted before
                // the row is removed.
                metrics::DEPLOYMENTS_TOTAL
                    .with_label_values(&["BuildFailed"])
                    .inc();
                self.emit(
                    DeploymentEvent::new(deployment_id, &subdomain, EventType::BuildFailed)
                        .with_duration_ms(build_start.elapsed().as_millis() as i64),
                )
                .await;
                drop(log_tx);
                let _ = log_drain.await;
                let _ = self.state.deployments.delete(&deployment_id).await;
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

        self.finish_deploy(
            deployment_id,
            subdomain,
            project,
            artifact,
            env_vars,
            log_tx,
            log_drain,
        )
        .await
    }

    /// Redeploys an already-built artifact from a past deployment — no git
    /// clone, no nix build. Used for rollback.
    pub async fn redeploy_artifact(
        &self,
        deployment_id: Uuid,
        subdomain: String,
        artifact_path: PathBuf,
    ) -> Result<()> {
        let project = match self.state.projects.find_by_subdomain(&subdomain).await {
            Ok(Some(p)) => p,
            _ => {
                return Err(anyhow::anyhow!(
                    "Project not found for subdomain {}",
                    subdomain
                ));
            }
        };
        let env_vars = self.decrypt_env_vars(&project).await;
        let (log_tx, log_drain) = self.spawn_log_drain(deployment_id);
        let _ = log_tx.send(format!(
            "Rolling back to previously built artifact at {}",
            artifact_path.display()
        ));

        self.finish_deploy(
            deployment_id,
            subdomain,
            project,
            artifact_path,
            env_vars,
            log_tx,
            log_drain,
        )
        .await
    }

    async fn decrypt_env_vars(&self, project: &ProjectRow) -> Option<Vec<String>> {
        let mut env_strings = Vec::new();
        if let Ok(env_rows) = self.state.projects.get_env_vars(project.id).await {
            let key_str = std::env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "00000000000000000000000000000000".to_string());
            let mut key_bytes = [0u8; 32];
            let bytes = key_str.as_bytes();
            let len = bytes.len().min(32);
            key_bytes[..len].copy_from_slice(&bytes[..len]);

            for row in env_rows {
                if let Ok(plaintext) =
                    common::crypto::decrypt(&row.value_encrypted, &row.nonce, &key_bytes)
                {
                    env_strings.push(format!("{}={}", row.key, plaintext));
                }
            }
        }
        if env_strings.is_empty() {
            None
        } else {
            Some(env_strings)
        }
    }

    /// Steps shared by a fresh deploy (after its build produces an
    /// artifact) and a rollback (which already has one): build the image,
    /// start the container, activate the route, and clean up the
    /// previously-active container.
    #[allow(clippy::too_many_arguments)]
    async fn finish_deploy(
        &self,
        deployment_id: Uuid,
        subdomain: String,
        project: ProjectRow,
        artifact: PathBuf,
        env_vars: Option<Vec<String>>,
        log_tx: common::buildlog::LogSender,
        log_drain: tokio::task::JoinHandle<()>,
    ) -> Result<()> {
        use domain::deployment::DeploymentStatus;

        // --- 3. Image Building & Container Starting ---
        tracing::info!("Build Completed. Building Image...");
        let _ = self
            .state
            .deployments
            .update_status(&deployment_id, DeploymentStatus::ImageBuilding)
            .await;

        let _ = self
            .state
            .deployments
            .update_status(&deployment_id, DeploymentStatus::ContainerStarting)
            .await;
        let (container_id, port) = match self
            .state
            .runtime
            .deploy(artifact, deployment_id, env_vars, Some(log_tx.clone()))
            .await
        {
            Ok(res) => res,
            Err(e) => {
                // Same reasoning as the BuildFailed case above: this
                // deployment never became Running, so it isn't kept.
                metrics::DEPLOYMENTS_TOTAL
                    .with_label_values(&["Crashed"])
                    .inc();
                self.emit(DeploymentEvent::new(
                    deployment_id,
                    &subdomain,
                    EventType::DeploymentCrashed,
                ))
                .await;
                drop(log_tx);
                let _ = log_drain.await;
                let _ = self.state.deployments.delete(&deployment_id).await;
                return Err(e.into());
            }
        };
        drop(log_tx);
        let _ = log_drain.await;
        metrics::ACTIVE_CONTAINERS.inc();
        self.emit(DeploymentEvent::new(
            deployment_id,
            &subdomain,
            EventType::ContainerStarted,
        ))
        .await;

        // --- 4. Running & Route Setup ---
        tracing::info!("{} container started", container_id);
        self.state.proxy.add_route(subdomain.clone(), port);

        if let Err(e) = self
            .state
            .deployments
            .set_container_info(&deployment_id, &container_id, port)
            .await
        {
            tracing::warn!("Failed to persist container info: {}", e);
        }
        let _ = self
            .state
            .deployments
            .update_status(&deployment_id, DeploymentStatus::Running)
            .await;
        // Also update the active deployment id for the project
        let _ = self
            .state
            .projects
            .update_active_deployment(&project.id, &deployment_id)
            .await;
        tracing::info!("Route activated {} -> {}", subdomain, port);
        metrics::DEPLOYMENTS_TOTAL
            .with_label_values(&["Running"])
            .inc();
        self.emit(DeploymentEvent::new(
            deployment_id,
            &subdomain,
            EventType::RouteActivated,
        ))
        .await;
        self.emit(DeploymentEvent::new(
            deployment_id,
            &subdomain,
            EventType::DeploymentCompleted,
        ))
        .await;

        // --- 5. Clean up old container ---
        if let Some(old_deployment_id) = project.active_deployment_id
            && let Ok(Some(old_deployment)) =
                self.state.deployments.find_by_id(&old_deployment_id).await
            && let Some(old_container_id) = old_deployment.container_id
        {
            tracing::info!("Stopping old container {}...", old_container_id);
            let _ = self.state.runtime.stop(&old_container_id).await;
            let _ = self.state.runtime.remove(&old_container_id).await;
            let _ = self
                .state
                .deployments
                .update_status(&old_deployment_id, DeploymentStatus::Stopped)
                .await;
            metrics::ACTIVE_CONTAINERS.dec();
        }

        Ok(())
    }
}
