pub mod router;
pub mod handlers;
pub mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use anyhow::Context;
    use sqlx::postgres::PgPoolOptions;
    
    use db::deployment_repo::DeploymentRepository;
    use builder::Builder;
    use runtime::runtime::Runtime;
    use proxy::state::ProxyState;
    use controller::state::ControlState;
    use controller::main_controller::ControlPlane;
    use common::logging;
    use crate::router::create_router;
    use crate::state::AppState;

    logging::init();
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
    let builder_path = std::path::PathBuf::from("/var/oxide/builds");
    let builder = Arc::new(Builder::new(builder_path));
    
    let runtime_path = std::path::PathBuf::from("/var/oxide/runtime");
    let runtime = Arc::new(Runtime::new(runtime_path).context("Failed to initialize Docker runtime")?);
    let proxy_state = Arc::new(ProxyState::new());

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
    };

    let health_state = Arc::new(control_state.clone());
    tokio::spawn(async move {
        controller::health_monitor::start_health_monitor(health_state).await;
    });

    let control_plane = Arc::new(ControlPlane::new(control_state));
    let api_state = AppState { control_plane };
    let app = create_router(api_state);

    let proxy_state_clone = proxy_state.clone();
    let proxy_handle = tokio::spawn(async move {
        tracing::info!("Starting Pingora Proxy Server...");
        // proxy::start_proxy blocks the thread, so run it in spawn_blocking
        let state_clone = proxy_state_clone.clone();
        tokio::task::spawn_blocking(move || {
            proxy::proxy::start_proxy((*state_clone).clone());
        }).await.unwrap();
    });

    let api_handle = tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:3001").await.expect("Failed to bind API port");
        tracing::info!("API Server listening on 0.0.0.0:3001");
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("API Server error: {}", e);
        }
    });

    tokio::select! {
        res = proxy_handle => {
            tracing::error!("Proxy server terminated: {:?}", res);
        }
        res = api_handle => {
            tracing::error!("API server terminated: {:?}", res);
        }
    };

    Ok(())
}
