use std::{path::Path};
use tokio::process::Command;

use common::error::OxideError;



pub async fn build(dir:&Path)->Result<(),OxideError>{
    let output = Command::new("nix")
        .arg("build")
        .current_dir(dir)
        .output()
        .await.map_err(|e| OxideError::Build(e.to_string()))?;
    if !output.status.success(){
        return Err(OxideError::Build(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    Ok(())
}
