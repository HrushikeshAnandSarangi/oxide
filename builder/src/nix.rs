use std::path::Path;
use tokio::process::Command;

use common::error::OxideError;

/// `impure` disables the build sandbox (`--option sandbox false`) — only
/// needed for the generated Python flake's network-dependent `pip install`.
/// Note this requires the invoking user to be a trusted user in the Nix
/// daemon's config for a multi-user install; otherwise the daemon silently
/// ignores the override and the build stays sandboxed.
pub async fn build(dir: &Path, impure: bool) -> Result<(), OxideError> {
    let mut cmd = Command::new("nix");
    cmd.arg("build").current_dir(dir);
    if impure {
        cmd.args(["--option", "sandbox", "false"]);
    }
    let output = cmd
        .output()
        .await
        .map_err(|e| OxideError::Build(e.to_string()))?;
    if !output.status.success() {
        return Err(OxideError::Build(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(())
}
