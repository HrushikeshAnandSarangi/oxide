use std::path::{PathBuf};

use common::error::OxideError;
use uuid::Uuid;

use crate::{artifact, git, nix};

pub struct Builder{
    pub root:PathBuf,
}
impl Builder {
    pub fn new(root:PathBuf)->Self{
        Self{root}
    }
    pub async fn build(&self,repo_url:&str,deployment_id:Uuid)->Result<PathBuf,OxideError>{
        let build_dir=self.root.join(deployment_id.to_string());
        git::clone(repo_url,&build_dir).await?;
        nix::build(&build_dir).await?;
        let artifact_path=artifact::resolve(&build_dir)?;
        Ok(artifact_path)
    }
    
}
