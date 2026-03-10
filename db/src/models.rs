use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ProjectEnvVarRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub key: String,
    pub value_encrypted: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ProjectRow{
    pub id:Uuid,
    pub name:String,
    pub subdomain:String,
    pub repo_url:Option<String>,
    pub install_command:Option<String>,
    pub build_command:Option<String>,
    pub run_command:Option<String>,
    pub root_directory:Option<String>,
    pub active_deployment_id:Option<Uuid>,
    pub created_at:DateTime<Utc>
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DeploymentRow{
    pub id: Uuid,
    pub project_id:Uuid,
    pub version:String,
    pub artifact_path:Option<String>,
    pub docker_image:Option<String>,
    pub container_id:Option<String>,
    pub status:String,
    pub created_at:DateTime<Utc>,
}

