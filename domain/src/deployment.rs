use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::types::{DeploymentId, ProjectId};



use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Queued,
    Building,
    BuildFailed,
    ImageBuilding,
    ContainerStarting,
    Running,
    Crashed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: DeploymentId,
    pub project_id: ProjectId,
    pub version: String,
    pub artifact_path: Option<String>,
    pub docker_image: Option<String>,
    pub container_id: Option<String>,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
}

impl Deployment {
    pub fn new(project_id:ProjectId,version:String)->Self{
        Self{
            id:Uuid::new_v4(),
            project_id,
            version,
            artifact_path:None,
            docker_image:None,
            container_id:None,
            status:DeploymentStatus::Building,
            created_at: Utc::now(),
        }
    }
    pub fn mark_build_failed(&mut self){
        self.status=DeploymentStatus::BuildFailed;
    }

    pub fn mark_running(&mut self){
        self.status=DeploymentStatus::Running;
    }

    pub fn mark_crashed(&mut self){
        self.status=DeploymentStatus::Crashed;
    }
}
