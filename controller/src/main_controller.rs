use crate::deployment_service::DeploymentService;
use crate::state::ControlState;
use anyhow::Result;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct ControlPlane {
    pub state: Arc<ControlState>,
    deployment_tx: tokio::sync::mpsc::Sender<(uuid::Uuid, String)>,
}

impl ControlPlane {
    /// `shutdown` stops the worker from picking up *new* queued deploys once
    /// cancelled, but never interrupts a deploy that's already running
    /// (nix build / docker build mid-flight is not something to abort
    /// abruptly — that risks stray containers or half-built images).
    pub fn new(state: ControlState, shutdown: CancellationToken) -> Self {
        let state = Arc::new(state);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(uuid::Uuid, String)>(100);

        let worker_state = state.clone();
        tokio::spawn(async move {
            tracing::info!("Async Deployment worker started.");
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown.cancelled() => {
                        tracing::info!("Deployment worker shutting down, no longer accepting new deploys");
                        break;
                    }
                    item = rx.recv() => {
                        let Some((deployment_id, subdomain)) = item else {
                            break;
                        };
                        let service = DeploymentService::new(worker_state.clone());
                        if let Err(e) = service.deploy(deployment_id, subdomain.clone()).await {
                            tracing::error!("Deployment pipeline failed for {}: {}", subdomain, e);
                        }
                    }
                }
            }
        });

        Self {
            state,
            deployment_tx: tx,
        }
    }

    pub async fn deploy(&self, project_subdomain: String) -> Result<uuid::Uuid> {
        let project = self
            .state
            .projects
            .find_by_subdomain(&project_subdomain)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        let deployment_id = uuid::Uuid::new_v4();
        let deployment = domain::Deployment {
            id: deployment_id,
            project_id: project.id,
            version: "v1".to_string(), // In reality we'd extract github sha here later
            artifact_path: None,
            docker_image: None,
            container_id: None,
            status: domain::deployment::DeploymentStatus::Queued,
            created_at: chrono::Utc::now(),
        };
        self.state.deployments.create(&deployment).await?;

        self.deployment_tx
            .send((deployment_id, project_subdomain))
            .await
            .map_err(|e| anyhow::anyhow!("Queue full or closed: {}", e))?;
        Ok(deployment_id)
    }

    /// Gracefully tears down a deployment: stops and removes its container
    /// (if it has one), removes its proxy route and clears the project's
    /// active_deployment_id (only if this deployment is actually the one
    /// currently routed — deleting an old, already-superseded deployment
    /// must never rip out a newer one's live route), then deletes the row.
    ///
    /// Deleting a deployment that's still mid-build (Queued/Building/etc.)
    /// doesn't cancel it — there's no cancellation hook into the builder
    /// pipeline today — it just removes the row; if that build later
    /// succeeds, its container ends up untracked. Acceptable for now since
    /// deleting something still in progress is expected to be rare.
    pub async fn delete_deployment(&self, deployment_id: uuid::Uuid) -> Result<()> {
        let deployment = self
            .state
            .deployments
            .find_by_id(&deployment_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Deployment not found"))?;

        if let Some(container_id) = &deployment.container_id {
            let _ = self.state.runtime.stop(container_id).await;
            let _ = self.state.runtime.remove(container_id).await;
        }

        if let Some(project) = self
            .state
            .projects
            .find_by_id(&deployment.project_id)
            .await?
            && project.active_deployment_id == Some(deployment_id)
        {
            self.state.proxy.remove_route(&project.subdomain);
            self.state
                .projects
                .clear_active_deployment(&project.id)
                .await?;
        }

        self.state.deployments.delete(&deployment_id).await?;
        Ok(())
    }

    pub async fn create_project(
        &self,
        project: &domain::Project,
        env_vars: Option<std::collections::HashMap<String, String>>,
    ) -> Result<()> {
        self.state.projects.create(project).await?;

        if let Some(vars) = env_vars {
            let key_str = std::env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "00000000000000000000000000000000".to_string());
            let mut key_bytes = [0u8; 32];
            let bytes = key_str.as_bytes();
            let len = bytes.len().min(32);
            key_bytes[..len].copy_from_slice(&bytes[..len]);

            for (k, v) in vars {
                if let Ok((ciphertext, nonce)) = common::crypto::encrypt(&v, &key_bytes) {
                    let env_var = domain::project::ProjectEnvVar {
                        id: uuid::Uuid::new_v4(),
                        project_id: project.id,
                        key: k,
                        value_encrypted: ciphertext,
                        nonce,
                        created_at: chrono::Utc::now(),
                    };
                    if let Err(e) = self.state.projects.add_env_var(&env_var).await {
                        tracing::error!("Failed to save env var: {}", e);
                    }
                }
            }
        }

        Ok(())
    }
}
