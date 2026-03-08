use chrono::{DateTime, Utc};
use uuid::Uuid;


#[derive(sqlx::FromRow)]
pub struct ProjectRow{
    pub id:Uuid,
    pub name:String,
    pub subdomain:String,
    pub active_deployment_id:Option<Uuid>,
    pub created_at:DateTime<Utc>
}

#[derive(sqlx::FromRow)]
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

