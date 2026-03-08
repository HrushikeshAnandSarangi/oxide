use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::types::{ProjectId, Subdomain};



#[derive(Debug,Clone)]
pub struct Project{
    pub id:ProjectId,
    pub name:String,
    pub subdomain:Subdomain,
    pub active_deployment_id:Option<Uuid>,
    pub created_at:DateTime<Utc>,
}

impl Project {
    pub fn new(name:String,subdomain:Subdomain)->Self{
        Self{
            id:Uuid::new_v4(),
            name,
            subdomain,
            active_deployment_id:None,
            created_at:Utc::now(),
            
        }
    }

    pub fn activate_deployment(&mut self,deployment_id:Uuid){
        self.active_deployment_id=Some(deployment_id);
    }
    
}
