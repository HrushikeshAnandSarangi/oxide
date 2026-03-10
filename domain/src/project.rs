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



#[derive(Debug,Clone)]
pub struct Project{
    pub id:ProjectId,
    pub name:String,
    pub subdomain:Subdomain,
    pub repo_url:Option<String>,
    pub install_command:Option<String>,
    pub build_command:Option<String>,
    pub run_command:Option<String>,
    pub root_directory:Option<String>,
    pub active_deployment_id:Option<Uuid>,
    pub created_at:DateTime<Utc>,
}

impl Project {
    pub fn new(name:String,subdomain:Subdomain, repo_url:Option<String>, install_command:Option<String>, build_command:Option<String>, run_command:Option<String>, root_directory:Option<String>) -> Self {
        Self{
            id:Uuid::new_v4(),
            name,
            subdomain,
            repo_url,
            install_command,
            build_command,
            run_command,
            root_directory,
            active_deployment_id:None,
            created_at:Utc::now(),
        }
    }

    pub fn activate_deployment(&mut self,deployment_id:Uuid){
        self.active_deployment_id=Some(deployment_id);
    }
    
}
