use std::{fs, path::{Path, PathBuf}};

use common::error::OxideError;



pub fn resolve(build_dir:&Path)->Result<PathBuf,OxideError>{
    let result_path=build_dir.join("result");

    if !result_path.exists(){
        return Err(OxideError::Build("No build result found".into()));
    }

    let target=fs::read_link(&result_path).map_err(|e|OxideError::Build(e.to_string()))?;
    Ok(target)
}
