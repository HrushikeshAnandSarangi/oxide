use std::{
    fs,
    path::{Path, PathBuf},
};

use common::error::OxideError;

pub fn resolve(build_dir: &Path) -> Result<PathBuf, OxideError> {
    let result_path = build_dir.join("result");

    if !result_path.exists() {
        return Err(OxideError::Build("No build result found".into()));
    }

    let target = fs::read_link(&result_path).map_err(|e| OxideError::Build(e.to_string()))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("oxide-artifact-test-{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_follows_the_result_symlink() {
        let build_dir = scratch_dir("resolve-ok");
        let nix_store_path = build_dir.join("fake-store-path");
        fs::create_dir_all(&nix_store_path).unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(&nix_store_path, build_dir.join("result")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&nix_store_path, build_dir.join("result")).unwrap();

        let resolved = resolve(&build_dir).unwrap();
        assert_eq!(resolved, nix_store_path);
    }

    #[test]
    fn resolve_errors_when_no_result_link_exists() {
        let build_dir = scratch_dir("resolve-missing");
        let err = resolve(&build_dir).unwrap_err();
        assert!(matches!(err, OxideError::Build(_)));
    }
}
