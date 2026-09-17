use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Go,
    TypeScript,
    JavaScript,
    Python,
}

/// Best-effort language detection from a repo's file layout. Checked in an
/// order where more specific signals (tsconfig.json) are tested before more
/// general ones (package.json alone) that would otherwise shadow them.
pub fn detect(repo_dir: &Path) -> Option<Language> {
    if repo_dir.join("Cargo.toml").is_file() {
        return Some(Language::Rust);
    }
    if repo_dir.join("go.mod").is_file() {
        return Some(Language::Go);
    }
    if repo_dir.join("package.json").is_file() {
        if repo_dir.join("tsconfig.json").is_file() {
            return Some(Language::TypeScript);
        }
        return Some(Language::JavaScript);
    }
    if repo_dir.join("pyproject.toml").is_file() || repo_dir.join("requirements.txt").is_file() {
        return Some(Language::Python);
    }
    None
}

/// The trigger condition described by the product requirement: only
/// auto-generate a flake for a repo that clearly wants to be containerized
/// (has a Dockerfile) but hasn't opted into Nix yet (no flake.nix already).
/// An existing flake.nix is always respected and never overwritten.
pub fn should_generate_flake(repo_dir: &Path) -> bool {
    repo_dir.join("Dockerfile").is_file() && !repo_dir.join("flake.nix").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("oxide-detect-test-{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn detects_rust_from_cargo_toml() {
        let dir = scratch_dir("rust");
        fs::write(dir.join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        assert_eq!(detect(&dir), Some(Language::Rust));
    }

    #[test]
    fn detects_go_from_go_mod() {
        let dir = scratch_dir("go");
        fs::write(dir.join("go.mod"), "module example.com/x").unwrap();
        assert_eq!(detect(&dir), Some(Language::Go));
    }

    #[test]
    fn detects_typescript_over_javascript_when_tsconfig_present() {
        let dir = scratch_dir("ts");
        fs::write(dir.join("package.json"), "{}").unwrap();
        fs::write(dir.join("tsconfig.json"), "{}").unwrap();
        assert_eq!(detect(&dir), Some(Language::TypeScript));
    }

    #[test]
    fn detects_javascript_without_tsconfig() {
        let dir = scratch_dir("js");
        fs::write(dir.join("package.json"), "{}").unwrap();
        assert_eq!(detect(&dir), Some(Language::JavaScript));
    }

    #[test]
    fn detects_python_from_requirements_txt() {
        let dir = scratch_dir("py");
        fs::write(dir.join("requirements.txt"), "flask").unwrap();
        assert_eq!(detect(&dir), Some(Language::Python));
    }

    #[test]
    fn returns_none_for_unrecognized_layout() {
        let dir = scratch_dir("none");
        assert_eq!(detect(&dir), None);
    }

    #[test]
    fn generates_only_when_dockerfile_present_and_flake_absent() {
        let dir = scratch_dir("trigger");
        assert!(!should_generate_flake(&dir));

        fs::write(dir.join("Dockerfile"), "FROM scratch").unwrap();
        assert!(should_generate_flake(&dir));

        fs::write(dir.join("flake.nix"), "{}").unwrap();
        assert!(!should_generate_flake(&dir));
    }
}
