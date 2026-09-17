pub mod handlers;
pub mod router;
pub mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use tokio::net::TcpListener;

    use crate::router::create_router;
    use crate::state::AppState;
    use builder::Builder;
    use common::logging;
    use controller::main_controller::ControlPlane;
    use controller::state::ControlState;
    use db::deployment_repo::DeploymentRepository;
    use proxy::state::ProxyState;
    use runtime::runtime::Runtime;

    dotenvy::dotenv().ok();
    logging::init();
    controller::metrics::init();
    tracing::info!("Starting Oxide Platform...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/oxide".to_string());

    tracing::info!("Initializing database connection pool...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to the database")?;

    let deployment_repo = Arc::new(DeploymentRepository::new(pool.clone()));
    let project_repo = Arc::new(db::project_repo::ProjectRepository::new(pool.clone()));

    tracing::info!("Initializing subsystems (Builder, Runtime, Proxy)...");
    // /var/oxide/* is the production default (see systemd/oxide-api.service,
    // which overrides these explicitly) — it needs root to create, so it
    // silently broke every local dev deploy with a "Permission denied"
    // buried inside a git-clone failure. Default to a repo-local directory
    // instead; production keeps working via the env var override.
    let builder_path = std::env::var("OXIDE_BUILDS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(".oxide/builds"));
    std::fs::create_dir_all(&builder_path).with_context(|| {
        format!(
            "Failed to create builds directory at {}",
            builder_path.display()
        )
    })?;
    let builder = Arc::new(Builder::new(builder_path));

    let runtime_path = std::env::var("OXIDE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(".oxide/runtime"));
    std::fs::create_dir_all(&runtime_path).with_context(|| {
        format!(
            "Failed to create runtime directory at {}",
            runtime_path.display()
        )
    })?;
    let runtime =
        Arc::new(Runtime::new(runtime_path).context("Failed to initialize Docker runtime")?);
    let proxy_state = Arc::new(ProxyState::new());

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let events = match common::events::EventPublisher::connect(&redis_url).await {
        Ok(publisher) => {
            tracing::info!("Connected to Redis for telemetry at {}", redis_url);
            Some(Arc::new(publisher))
        }
        Err(e) => {
            tracing::warn!("Telemetry disabled — could not connect to Redis: {}", e);
            None
        }
    };

    tracing::info!("Recovering proxy routing state from database...");
    match deployment_repo.get_running_deployments().await {
        std::result::Result::Ok(active_routes) => {
            for (subdomain, port) in active_routes {
                proxy_state.add_route(subdomain.clone(), port);
                tracing::info!("Recovered route: {} -> {}", subdomain, port);
            }
        }
        Err(e) => tracing::warn!("Failed to recover routing state: {}", e),
    }

    let control_state = ControlState {
        builder,
        runtime,
        proxy: proxy_state.clone(),
        deployments: deployment_repo,
        projects: project_repo,
        events,
    };

    // Cancelled once an OS shutdown signal arrives; every long-running task
    // below watches it to stop accepting new work and wind down cleanly.
    // The Pingora proxy is the one exception — it installs its own
    // SIGTERM/SIGINT handlers internally (see common::shutdown::wait_for_signal
    // for why) and isn't wired to this token.
    let shutdown = tokio_util::sync::CancellationToken::new();

    let health_state = Arc::new(control_state.clone());
    let health_shutdown = shutdown.clone();
    let health_handle = tokio::spawn(async move {
        controller::health_monitor::start_health_monitor(health_state, health_shutdown).await;
    });

    let control_plane = Arc::new(ControlPlane::new(control_state, shutdown.clone()));
    let api_state = AppState { control_plane };
    let app = create_router(api_state);

    let proxy_state_clone = proxy_state.clone();
    let proxy_handle = tokio::spawn(async move {
        tracing::info!("Starting Pingora Proxy Server...");
        // proxy::start_proxy blocks the thread, so run it in spawn_blocking
        let state_clone = proxy_state_clone.clone();
        tokio::task::spawn_blocking(move || {
            proxy::proxy::start_proxy((*state_clone).clone());
        })
        .await
        .unwrap();
        tracing::info!("Pingora proxy server has shut down");
    });

    let api_shutdown = shutdown.clone();
    let api_handle = tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:3001")
            .await
            .expect("Failed to bind API port");
        tracing::info!("API Server listening on 0.0.0.0:3001");
        // `with_graceful_shutdown`'s future runs the server indefinitely —
        // accepting and serving requests — until `api_shutdown` is
        // cancelled, and only then drains in-flight requests. It must be
        // awaited directly, not wrapped in a timeout: a timeout here bounds
        // the server's *entire* lifetime, not just the post-shutdown drain
        // (this previously killed the API after a flat 20s regardless of
        // whether shutdown was ever requested).
        match axum::serve(listener, app)
            .with_graceful_shutdown(async move { api_shutdown.cancelled().await })
            .await
        {
            Ok(()) => tracing::info!("API server has shut down"),
            Err(e) => tracing::error!("API server error: {}", e),
        }
    });

    // Trigger cancellation on SIGTERM/Ctrl+C, then wait for everything that
    // depends on it to actually finish before this process exits.
    tokio::spawn({
        let shutdown = shutdown.clone();
        async move {
            common::shutdown::wait_for_signal().await;
            tracing::info!("Shutdown signal received, starting graceful shutdown...");
            shutdown.cancel();
        }
    });

    let _ = tokio::join!(api_handle, proxy_handle, health_handle);
    tracing::info!("Oxide has shut down");

    Ok(())
}
