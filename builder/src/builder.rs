use std::path::{Path, PathBuf};

use common::error::OxideError;
use uuid::Uuid;

use crate::{artifact, detect, flake_gen, git, nix};

pub struct Builder {
    pub root: PathBuf,
}
impl Builder {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub async fn build(
        &self,
        repo_url: &str,
        deployment_id: Uuid,
        auto_generate_flake: bool,
    ) -> Result<PathBuf, OxideError> {
        let build_dir = self.root.join(deployment_id.to_string());
        git::clone(repo_url, &build_dir).await?;

        let impure = self
            .maybe_generate_flake(&build_dir, auto_generate_flake)
            .await?;
        nix::build(&build_dir, impure).await?;

        let artifact_path = artifact::resolve(&build_dir)?;
        Ok(artifact_path)
    }

    /// If the project opted in (`auto_generate_flake`) and the repo looks
    /// like a candidate (has a Dockerfile, no flake.nix yet), generates one
    /// for the detected language. Go/JS/TS need a dependency-vendor hash
    /// Nix can't compute up front, so this makes one throwaway build
    /// attempt with a placeholder hash, extracts the real hash from Nix's
    /// own error output, and patches it in — the caller's own `nix::build`
    /// afterward is what actually produces the artifact.
    ///
    /// Returns whether that subsequent build needs sandboxing disabled
    /// (only true for the Python/pip path).
    async fn maybe_generate_flake(
        &self,
        build_dir: &Path,
        auto_generate_flake: bool,
    ) -> Result<bool, OxideError> {
        if !auto_generate_flake || !detect::should_generate_flake(build_dir) {
            return Ok(false);
        }

        let Some(lang) = detect::detect(build_dir) else {
            tracing::warn!(
                "auto_generate_flake is set but the repo's language couldn't be detected; \
                 falling back to the repo's own build setup"
            );
            return Ok(false);
        };

        tracing::info!(
            "Auto-generating a Nix flake for detected language: {:?}",
            lang
        );
        let generated = flake_gen::generate(build_dir, lang)?;
        let impure = matches!(lang, detect::Language::Python);

        if generated.needs_hash_retry
            && let Err(OxideError::Build(stderr)) = nix::build(build_dir, impure).await
        {
            match flake_gen::extract_real_hash(&stderr) {
                Some(real_hash) => {
                    tracing::info!(
                        "Discovered dependency hash from Nix's error output, patching and retrying"
                    );
                    flake_gen::patch_hash(build_dir, &real_hash)?;
                }
                None => return Err(OxideError::Build(stderr)),
            }
        }

        Ok(impure)
    }
}
