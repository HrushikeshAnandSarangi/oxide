use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::{
    deploy::{delete_deployment, deploy, rollback},
    health::health,
    history::{
        get_deployment, get_deployment_logs, get_project_logs, list_deployments, list_projects,
    },
    metrics::metrics,
    project::create_project,
};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/deploy", post(deploy))
        .route("/rollback", post(rollback))
        .route("/project", post(create_project))
        .route("/projects", get(list_projects))
        .route("/projects/{id}/logs", get(get_project_logs))
        .route("/deployments", get(list_deployments))
        .route(
            "/deployments/{id}",
            get(get_deployment).delete(delete_deployment),
        )
        .route("/deployments/{id}/logs", get(get_deployment_logs))
        .with_state(state)
}
