use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProjectEnvVar {
    pub id: Uuid,
    pub project_id: Uuid,
    pub key: String,
    pub value_encrypted: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

use crate::types::{ProjectId, Subdomain};

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub subdomain: Subdomain,
    pub repo_url: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
    pub run_command: Option<String>,
    pub root_directory: Option<String>,
    /// Opt-in: if the repo has a Dockerfile but no flake.nix, generate one
    /// automatically at build time (see builder::detect / builder::flake_gen).
    /// Off by default — this is a deliberate choice by whoever deploys the
    /// project, not something Oxide does just because it noticed a Dockerfile.
    pub auto_generate_flake: bool,
    pub active_deployment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Project {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        subdomain: Subdomain,
        repo_url: Option<String>,
        install_command: Option<String>,
        build_command: Option<String>,
        run_command: Option<String>,
        root_directory: Option<String>,
        auto_generate_flake: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            subdomain,
            repo_url,
            install_command,
            build_command,
            run_command,
            root_directory,
            auto_generate_flake,
            active_deployment_id: None,
            created_at: Utc::now(),
        }
    }

    pub fn activate_deployment(&mut self, deployment_id: Uuid) {
        self.active_deployment_id = Some(deployment_id);
    }
}
