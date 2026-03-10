use std::path::PathBuf;

use bollard::{Docker};
use common::error::OxideError;
use uuid::Uuid;

use crate::{container, image};

pub struct Runtime{
    docker: Docker,
    pub root:PathBuf,
}
impl Runtime {
    pub fn new(root:PathBuf)->Result<Self,OxideError>{
        let docker=Docker::connect_with_local_defaults().map_err(|e|OxideError::Runtime(e.to_string()))?;
        Ok(Self{docker,root})
    }
    pub async fn deploy(&self,artifact_path:PathBuf,deployment_id:Uuid, env_vars: Option<Vec<String>>)->Result<(String,u16),OxideError>{
        let image_tag=format!("oxide-{}",deployment_id);
        image::build(&self.docker,&image_tag,artifact_path).await?;
        let (container_id,port)=container::run_container(&self.docker,&image_tag, env_vars).await?;
        Ok((container_id,port))
    }

    pub async fn stop(&self, id: &str) -> Result<(), OxideError> {
        container::stop_container(&self.docker, id).await
    }

    pub async fn remove(&self, id: &str) -> Result<(), OxideError> {
        container::remove_container(&self.docker, id).await
    }
}
