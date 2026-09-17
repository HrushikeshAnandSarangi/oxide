use std::path::Path;
use std::process::Stdio;

use common::buildlog::LogSender;
use common::error::OxideError;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// `impure` disables the build sandbox (`--option sandbox false`) — only
/// needed for the generated Python flake's network-dependent `pip install`.
/// Note this requires the invoking user to be a trusted user in the Nix
/// daemon's config for a multi-user install; otherwise the daemon silently
/// ignores the override and the build stays sandboxed.
///
/// `log_tx`, if given, receives each line of `nix build`'s stdout/stderr as
/// it's produced — lets a caller show "how it's building" live instead of
/// only finding out the outcome once the whole build finishes.
pub async fn build(dir: &Path, impure: bool, log_tx: Option<LogSender>) -> Result<(), OxideError> {
    let mut cmd = Command::new("nix");
    cmd.arg("build")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if impure {
        cmd.args(["--option", "sandbox", "false"]);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| OxideError::Build(format!("failed to spawn nix build: {e}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| OxideError::Build("nix build: no stdout handle".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| OxideError::Build("nix build: no stderr handle".to_string()))?;

    let stdout_tx = log_tx.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(tx) = &stdout_tx {
                let _ = tx.send(line);
            }
        }
    });

    let stderr_tx = log_tx.clone();
    let stderr_task = tokio::spawn(async move {
        let mut captured = String::new();
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(tx) = &stderr_tx {
                let _ = tx.send(line.clone());
            }
            captured.push_str(&line);
            captured.push('\n');
        }
        captured
    });

    let (_stdout_res, stderr_res, wait_res) = tokio::join!(stdout_task, stderr_task, child.wait());

    let captured_stderr = stderr_res.unwrap_or_default();
    let status = wait_res.map_err(|e| OxideError::Build(e.to_string()))?;

    if !status.success() {
        return Err(OxideError::Build(captured_stderr));
    }
    Ok(())
}
