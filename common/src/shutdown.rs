/// Resolves when the process receives SIGTERM or Ctrl+C (SIGINT). Shared by
/// every Oxide binary so they all agree on what "please shut down" means.
///
/// Note: the Pingora proxy (spawned inside `api`) handles SIGTERM/SIGINT
/// itself internally (`run_forever()` installs its own signal watchers —
/// SIGTERM does a graceful drain, SIGINT is a fast shutdown). This function
/// is for the parts of Oxide Pingora doesn't own: the Axum API server and
/// the telemetry consumer.
pub async fn wait_for_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::warn!("Failed to install SIGTERM handler: {}", e);
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
