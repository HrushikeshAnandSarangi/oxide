use std::sync::Arc;
use anyhow::Result;
use crate::state::ControlState;
use crate::deployment_service::DeploymentService;

pub struct ControlPlane {
    pub state: Arc<ControlState>,
    deployment_tx: tokio::sync::mpsc::Sender<(uuid::Uuid, String)>,
}

impl ControlPlane {
    pub fn new(state: ControlState) -> Self {
        let state = Arc::new(state);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(uuid::Uuid, String)>(100);
        
        let worker_state = state.clone();
        tokio::spawn(async move {
            tracing::info!("Async Deployment worker started.");
            while let Some((deployment_id, subdomain)) = rx.recv().await {
                let service = DeploymentService::new(worker_state.clone());
                if let Err(e) = service.deploy(deployment_id, subdomain.clone()).await {
                    tracing::error!("Deployment pipeline failed for {}: {}", subdomain, e);
                }
            }
        });

        Self {
            state,
            deployment_tx: tx,
        }
    }

    pub async fn deploy(&self, project_subdomain: String) -> Result<uuid::Uuid> {
        let project = self.state.projects.find_by_subdomain(&project_subdomain).await?.ok_or_else(|| anyhow::anyhow!("Project not found"))?;
        
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
        
        self.deployment_tx.send((deployment_id, project_subdomain)).await.map_err(|e| anyhow::anyhow!("Queue full or closed: {}", e))?;
        Ok(deployment_id)
    }

    pub async fn create_project(&self, project: &domain::Project, env_vars: Option<std::collections::HashMap<String, String>>) -> Result<()> {
        self.state.projects.create(project).await?;
        
        if let Some(vars) = env_vars {
            let key_str = std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| "00000000000000000000000000000000".to_string());
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
